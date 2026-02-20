use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use lua_integration::{LuaIntegration, LuaKind};

use crate::{error::Error, providers::Provider};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub provider: Provider,
    pub choices: Vec<Choice>,
    #[serde(with = "time::serde::timestamp")]
    pub created: OffsetDateTime,
    pub model: String,
    pub object: Object,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<ResponseUsage>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaKind)]
#[repr(u8)]
pub enum Object {
    #[serde(rename = "chat.completion")]
    Completion = 1,
    #[serde(rename = "chat.completion.chunk")]
    Chunk,
}

/// Usage statistics.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct ResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cost: Option<u32>,
    pub prompt_tokens_details: Option<PromptTokenDetails>,
    pub completion_tokens_details: Option<CompletionTokenDetails>,
    pub total_tokens: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct PromptTokenDetails {
    pub cached_tokens: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct CompletionTokenDetails {
    pub reasoning_tokens: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaKind)]
#[serde(untagged)]
pub enum Choice {
    NonChat(NonChatChoice),
    NonStreaming(NonStreamingChoice),
    Streaming(StreamingChoice),
}

/// A non‑chat (plain text) choice.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct NonChatChoice {
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    pub text: String,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub error: Option<Error>,
}

/// A non‑streaming chat choice, which carries a complete message.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct NonStreamingChoice {
    // Could probably be an enum.
    #[serde(default)]
    pub finish_reason: Option<String>,
    #[serde(default)]
    pub native_finish_reason: Option<String>,
    pub index: usize,
    pub message: Message,
    #[serde(default)]
    pub error: Option<Error>,
}

/// A streaming chat choice, which carries a delta update.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct StreamingChoice {
    #[serde(default)]
    pub finish_reason: Option<String>,
    pub delta: Delta,
    #[serde(default)]
    pub error: Option<Error>,
}

/*
// Currently not implemented. Likely equal to this one:
// https://platform.openai.com/docs/api-reference/chat/object
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct LogProbs {
    pub content: Vec<()>,
    pub refusal: Vec<()>,
}
*/

/// A full message for non‑streaming choices.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct Message {
    pub role: String,
    #[serde(default)]
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub reasoning: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A delta update for streaming choices.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, LuaIntegration)]
pub struct Delta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, LuaIntegration)]
pub struct ToolCall {
    pub id: String,
    pub index: usize,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionCall,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, LuaIntegration)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use pretty_assertions::assert_eq;
    use time::{Date, Month, OffsetDateTime, Time};

    use super::*;

    #[test]
    fn object_round_trip() {
        let obj = Object::Completion;
        let serialized = serde_json::to_string(&obj).unwrap();
        assert_eq!(serialized, "\"chat.completion\"");
        let deserialized: Object = serde_json::from_str(&serialized).unwrap();
        assert_eq!(obj, deserialized);

        let obj = Object::Chunk;
        let serialized = serde_json::to_string(&obj).unwrap();
        assert_eq!(serialized, "\"chat.completion.chunk\"");
        let deserialized: Object = serde_json::from_str(&serialized).unwrap();
        assert_eq!(obj, deserialized);
    }

    #[test]
    fn completion_token_details_round_trip() {
        let ctd = CompletionTokenDetails {
            reasoning_tokens: 42,
        };
        let serialized = serde_json::to_string(&ctd).unwrap();
        assert_eq!(serialized, "{\"reasoning_tokens\":42}");
        let deserialized: CompletionTokenDetails = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ctd, deserialized);
    }

    #[test]
    fn prompt_token_details_round_trip() {
        let ptd = PromptTokenDetails {
            cached_tokens: 100,
        };
        let serialized = serde_json::to_string(&ptd).unwrap();
        assert_eq!(serialized, "{\"cached_tokens\":100}");
        let deserialized: PromptTokenDetails = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ptd, deserialized);
    }

    /*
    #[test]
    fn logprobs_round_trip() {
        let lp = LogProbs {
            content: vec![],
            refusal: vec![],
        };
        let serialized = serde_json::to_string(&lp).unwrap();
        assert_eq!(serialized, "{\"content\":[],\"refusal\":[]}");
        let deserialized: LogProbs = serde_json::from_str(&serialized).unwrap();
        assert_eq!(lp, deserialized);
    }
    */

    #[test]
    fn response_usage_round_trip() {
        let ru = ResponseUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            cost: Some(5),
            prompt_tokens_details: Some(PromptTokenDetails {
                cached_tokens: 3,
            }),
            completion_tokens_details: Some(CompletionTokenDetails {
                reasoning_tokens: 7,
            }),
            total_tokens: 30,
        };
        let serialized = serde_json::to_string(&ru).unwrap();
        let deserialized: ResponseUsage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ru, deserialized);

        let ru = ResponseUsage {
            prompt_tokens: 5,
            completion_tokens: 10,
            cost: None,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            total_tokens: 15,
        };
        let serialized = serde_json::to_string(&ru).unwrap();
        let deserialized: ResponseUsage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ru, deserialized);
    }

    #[test]
    fn delta_round_trip() {
        let delta = Delta {
            content: Some("Hello".to_string()),
            role: Some("assistant".to_string()),
            tool_calls: None,
        };
        let serialized = serde_json::to_string(&delta).unwrap();
        let deserialized: Delta = serde_json::from_str(&serialized).unwrap();
        assert_eq!(delta, deserialized);

        let delta = Delta {
            content: None,
            role: None,
            tool_calls: None,
        };
        let serialized = serde_json::to_string(&delta).unwrap();
        let deserialized: Delta = serde_json::from_str(&serialized).unwrap();
        assert_eq!(delta, deserialized);
    }

    #[test]
    fn message_round_trip() {
        let msg = Message {
            role: "assistant".to_string(),
            content: Some("Hello there".to_string()),
            refusal: None,
            reasoning: Some("thinking...".to_string()),
            tool_calls: None,
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);

        let msg = Message {
            role: "user".to_string(),
            content: None,
            refusal: Some("I can't do that".to_string()),
            reasoning: None,
            tool_calls: None,
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn non_chat_choice_round_trip() {
        let ncc = NonChatChoice {
            finish_reason: Some("stop".to_string()),
            native_finish_reason: None,
            text: "Generated text".to_string(),
            reasoning: None,
            error: None,
        };
        let serialized = serde_json::to_string(&ncc).unwrap();
        let deserialized: NonChatChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ncc, deserialized);
    }

    #[test]
    fn non_streaming_choice_round_trip() {
        let nsc = NonStreamingChoice {
            finish_reason: Some("stop".to_string()),
            native_finish_reason: None,
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: Some("Response".to_string()),
                refusal: None,
                reasoning: None,
                tool_calls: None,
            },
            error: None,
        };
        let serialized = serde_json::to_string(&nsc).unwrap();
        let deserialized: NonStreamingChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(nsc, deserialized);
    }

    #[test]
    fn streaming_choice_round_trip() {
        let sc = StreamingChoice {
            finish_reason: None,
            delta: Delta {
                content: Some("chunk".to_string()),
                role: None,
                tool_calls: None,
            },
            error: None,
        };
        let serialized = serde_json::to_string(&sc).unwrap();
        let deserialized: StreamingChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(sc, deserialized);
    }

    #[test]
    fn choice_round_trip() {
        let choice = Choice::NonStreaming(NonStreamingChoice {
            finish_reason: None,
            native_finish_reason: None,
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: Some("Hello".to_string()),
                refusal: None,
                reasoning: None,
                tool_calls: None,
            },
            error: None,
        });
        let serialized = serde_json::to_string(&choice).unwrap();
        let deserialized: Choice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(choice, deserialized);
    }

    #[test]
    fn response_round_trip() {
        use crate::providers::Provider;

        let response = Response {
            id: "gen-123".to_string(),
            provider: Provider::OpenAI,
            choices: vec![Choice::NonStreaming(NonStreamingChoice {
                finish_reason: Some("stop".to_string()),
                native_finish_reason: None,
                index: 0,
                message: Message {
                    role: "assistant".to_string(),
                    content: Some("Test response".to_string()),
                    refusal: None,
                    reasoning: None,
                    tool_calls: None,
                },
                error: None,
            })],
            created: OffsetDateTime::new_utc(
                Date::from_calendar_date(2025, Month::January, 1).unwrap(),
                Time::from_hms(0, 0, 0).unwrap(),
            ),
            model: "gpt-4".to_string(),
            object: Object::Completion,
            system_fingerprint: None,
            usage: Some(ResponseUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                cost: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                total_tokens: 30,
            }),
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&serialized).unwrap();
        assert_eq!(response, deserialized);
    }

    #[test]
    fn chat_completion() {
        let file = read_to_string("./responses/chat-completion.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Response {
            id: "gen-1738698054-yI7fNDyuNM2VdvsYZ6Po".to_string(),
            provider: Provider::GoogleAIStudio,
            choices: vec![Choice::NonStreaming(NonStreamingChoice {
                index: 0,
                finish_reason: None,
                native_finish_reason: None,
                message: Message {
                    content: Some("I am a large language model, trained by Google.".to_owned()),
                    role: "assistant".to_string(),
                    refusal: None,
                    reasoning: None,
                    tool_calls: None,
                },
                error: None,
            })],
            created: OffsetDateTime::new_utc(
                Date::from_calendar_date(2025, Month::February, 4).unwrap(),
                Time::from_hms(19, 40, 54).unwrap(),
            ),
            model: "google/gemini-2.0-flash-thinking-exp".to_string(),
            object: Object::Completion,
            system_fingerprint: None,
            usage: Some(ResponseUsage {
                prompt_tokens: 18,
                completion_tokens: 11,
                cost: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                total_tokens: 29,
            }),
        };

        assert_eq!(expected, response);
    }

    #[test]
    fn completion() {
        let file = read_to_string("./responses/completion.json").unwrap();
        let response: Response = serde_json::from_str(&file).unwrap();

        let expected = Response {
            id: "gen-1738697078-nbXebb9pscDuajFRYYob".to_string(),
            provider: Provider::Targon,
            choices: vec![Choice::NonChat(NonChatChoice {
                finish_reason: None,
                text: "This is a shortened text output".to_string(),
                reasoning: None,
                error: None,
                native_finish_reason: None,
            })],
            created: OffsetDateTime::new_utc(
                Date::from_calendar_date(2025, Month::February, 4).unwrap(),
                Time::from_hms(19, 24, 38).unwrap(),
            ),
            model: "deepseek/deepseek-r1-distill-llama-70b".to_string(),
            object: Object::Completion,
            system_fingerprint: None,
            usage: Some(ResponseUsage {
                prompt_tokens: 4,
                completion_tokens: 589,
                cost: None,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                total_tokens: 593,
            }),
        };

        assert_eq!(expected, response);
    }

    #[test]
    fn tool_call() {
        let file = read_to_string("./responses/completion/chat/2025-03-07-tool-call.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn reasoning() {
        let file = read_to_string("./responses/completion/chat/2025-03-07-reasoning.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn simple() {
        let file = read_to_string("./responses/completion/normal/2025-03-07.json").unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }

    #[test]
    fn unknown() {
        let file =
            read_to_string("./responses/completion/chat/2025-03-30-tools-and-reasoning.json")
                .unwrap();
        let _response: Response = serde_json::from_str(&file).unwrap();
    }
}
