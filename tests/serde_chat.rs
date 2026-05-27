#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Round-trips de désérialisation des schémas `/v1/chat/completions`.

use opengatellm::{
    ChatCompletion, ChatContent, ChatMessage, ContentPart, CreateChatCompletion, FinishReason,
    Role, Stop, ToolCall,
};

#[test]
fn serialize_minimal_request() {
    let req = CreateChatCompletion::new(
        vec![
            ChatMessage::system("Tu es un assistant utile."),
            ChatMessage::user("Bonjour ?"),
        ],
        "qwen3-coder",
    )
    .temperature(0.2)
    .max_completion_tokens(64);
    let v = serde_json::to_value(&req).unwrap();
    assert_eq!(v["model"], "qwen3-coder");
    assert_eq!(v["temperature"], 0.2);
    assert_eq!(v["max_completion_tokens"], 64);
    assert_eq!(v["messages"][0]["role"], "system");
    assert_eq!(v["messages"][1]["role"], "user");
    assert_eq!(v["messages"][1]["content"], "Bonjour ?");
    assert!(v.get("stream").is_none(), "stream None must be skipped");
    assert!(v.get("frequency_penalty").is_none());
}

#[test]
fn serialize_multimodal_user_message() {
    let msg = ChatMessage {
        role: Role::User,
        content: Some(ChatContent::Parts(vec![
            ContentPart::Text {
                text: "Décris l'image.".into(),
            },
            ContentPart::ImageUrl {
                image_url: opengatellm::ImageUrl {
                    url: "data:image/png;base64,iVBOR...".into(),
                    detail: Some("high".into()),
                },
            },
        ])),
        name: None,
        refusal: None,
        annotations: None,
        tool_call_id: None,
        tool_calls: None,
    };
    let v = serde_json::to_value(&msg).unwrap();
    assert_eq!(v["role"], "user");
    assert_eq!(v["content"][0]["type"], "text");
    assert_eq!(v["content"][0]["text"], "Décris l'image.");
    assert_eq!(v["content"][1]["type"], "image_url");
    assert_eq!(v["content"][1]["image_url"]["detail"], "high");
}

#[test]
fn serialize_stop_variants() {
    let s = serde_json::to_value(Stop::Single("END".into())).unwrap();
    assert_eq!(s, serde_json::json!("END"));
    let m = serde_json::to_value(Stop::Many(vec!["END".into(), "\n\n".into()])).unwrap();
    assert_eq!(m, serde_json::json!(["END", "\n\n"]));
}

const SAMPLE_COMPLETION: &str = r#"
{
  "id": "chatcmpl-abc",
  "object": "chat.completion",
  "created": 1700000000,
  "model": "qwen3-coder",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Bonjour ! Comment puis-je vous aider ?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 8,
    "total_tokens": 20,
    "impacts": {"kWh": 0.0001, "kgCO2eq": 0.00005}
  }
}
"#;

#[test]
fn deserialize_basic_completion() {
    let c: ChatCompletion = serde_json::from_str(SAMPLE_COMPLETION).unwrap();
    assert_eq!(c.id, "chatcmpl-abc");
    assert_eq!(c.model, "qwen3-coder");
    assert_eq!(c.choices.len(), 1);
    let ch = &c.choices[0];
    assert_eq!(ch.index, 0);
    assert_eq!(ch.finish_reason, FinishReason::Stop);
    assert_eq!(ch.message.role, Role::Assistant);
    let content = ch.message.content.as_ref().unwrap();
    let ChatContent::Text(s) = content else {
        panic!("expected text content");
    };
    assert!(s.contains("Bonjour"));
    let usage = c.usage.as_ref().unwrap();
    assert_eq!(usage.prompt_tokens, 12);
    assert_eq!(usage.completion_tokens, 8);
    let impacts = usage.impacts.as_ref().unwrap();
    assert!((impacts.kwh - 0.0001).abs() < 1e-9);
    assert!((impacts.kg_co2eq - 0.000_05).abs() < 1e-9);
}

const SAMPLE_TOOL_CALL: &str = r#"
{
  "id": "chatcmpl-xyz",
  "object": "chat.completion",
  "created": 1700000000,
  "model": "qwen3-coder",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "type": "function",
            "id": "call_1",
            "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ]
}
"#;

#[test]
fn deserialize_completion_with_tool_call() {
    let c: ChatCompletion = serde_json::from_str(SAMPLE_TOOL_CALL).unwrap();
    let ch = &c.choices[0];
    assert_eq!(ch.finish_reason, FinishReason::ToolCalls);
    assert!(ch.message.content.is_none());
    let tool_calls = ch.message.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1);
    match &tool_calls[0] {
        ToolCall::Function { id, function } => {
            assert_eq!(id, "call_1");
            assert_eq!(function.name, "get_weather");
            assert!(function.arguments.contains("Paris"));
        }
        ToolCall::Custom { .. } => panic!("expected function tool call"),
    }
}

#[test]
fn deserialize_completion_without_optional_fields() {
    let json = r#"
    {
      "choices": [
        {
          "index": 0,
          "message": {"role": "assistant", "content": "ok"},
          "finish_reason": "stop"
        }
      ],
      "created": 1,
      "model": "m"
    }
    "#;
    let c: ChatCompletion = serde_json::from_str(json).unwrap();
    assert!(c.id.is_empty());
    assert!(c.usage.is_none());
    assert!(c.system_fingerprint.is_none());
}
