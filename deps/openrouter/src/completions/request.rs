use std::collections::BTreeMap;

use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeMap, de::{self, Visitor, MapAccess}};
use serde_json::value::{Value};

use crate::{
    completions::response::ToolCall,
    providers::Provider,
    primatives::{*},
};

// type Request = {
//   // Either "messages" or "prompt" is required
//   messages?: Message[];
//   prompt?: string;
//
//   // If "model" is unspecified, uses the user's default
//   model?: string; // See "Supported Models" section
//
//   // Allows to force the model to produce specific output format.
//   // See models page and note on this docs page for which models support it.
//   response_format?: { type: 'json_object' };
//
//   stop?: string | string[];
//   stream?: boolean; // Enable streaming
//
//   // See LLM Parameters (openrouter.ai/docs/parameters)
//   max_tokens?: number; // Range: [1, context_length)
//   temperature?: number; // Range: [0, 2]
//
//   // Tool calling
//   // Will be passed down as-is for providers implementing OpenAI's interface.
//   // For providers with custom interfaces, we transform and map the properties.
//   // Otherwise, we transform the tools into a YAML template. The model responds with an assistant message.
//   // See models supporting tool calling: openrouter.ai/models?supported_parameters=tools
//   tools?: Tool[];
//   tool_choice?: ToolChoice;
//
//   // Advanced optional parameters
//   seed?: number; // Integer only
//   top_p?: number; // Range: (0, 1]
//   top_k?: number; // Range: [1, Infinity) Not available for OpenAI models
//   frequency_penalty?: number; // Range: [-2, 2]
//   presence_penalty?: number; // Range: [-2, 2]
//   repetition_penalty?: number; // Range: (0, 2]
//   logit_bias?: { [key: number]: number };
//   top_logprobs: number; // Integer only
//   min_p?: number; // Range: [0, 1]
//   top_a?: number; // Range: [0, 1]
//
//   // Reduce latency by providing the model with a predicted output
//   // https://platform.openai.com/docs/guides/latency-optimization#use-predicted-outputs
//   prediction?: { type: 'content'; content: string };
//
//   // OpenRouter-only parameters
//   // See "Prompt Transforms" section: openrouter.ai/docs/transforms
//   transforms?: string[];
//   // See "Model Routing" section: openrouter.ai/docs/model-routing
//   models?: string[];
//   route?: 'fallback';
//   // See "Provider Routing" section: openrouter.ai/docs/provider-routing
//   provider?: ProviderPreferences;
//
//   // Whether to return the model's reasoning. Default false.
//   // Text will appear in the "reasoning" field on each message prior to those containing "content".
//   include_reasoning?: boolean;
// };
/// Represents a request to the model.
/// Note that both `messages` and `prompt` are optional at the type level;
/// additional validation would be needed to ensure at least one is provided.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Hash)]
pub struct Request {
    /// Allows to define a chat history with various participants and mixed content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,

    /// Allows to define a raw prompt that the model will start completion at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// If "model" is unspecified, uses the user's default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Forces the model to produce a specific output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Allows defining one or more tokens for generation to stop at.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    /// Enable streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// LLM parameter: maximum tokens (Range: [1, context_length)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,

    /// LLM parameter: temperature (Range: [0, 2]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<Temperature>,

    /// Tool calling: tools will be passed down as-is (if supported).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Advanced optional parameters:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<TopP>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<FrequencyPenalty>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<PresencePenalty>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<RepetitionPenalty>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<BTreeMap<String, Bias>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<MinP>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<TopA>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Reduce latency by providing a predicted output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,

    /// OpenRouter-only parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Route>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,

    /// Whether to return the model's reasoning. Default is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Stop {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ResponseFormat {
    JsonObject = 1,
}

impl Serialize for ResponseFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseFormat::JsonObject => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "json_object")?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ResponseFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResponseFormatVisitor;

        impl<'de> Visitor<'de> for ResponseFormatVisitor {
            type Value = ResponseFormat;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with 'type' field set to 'json_object'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut type_value: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "type" {
                        type_value = Some(map.next_value()?);
                    } else {
                        map.next_value::<de::IgnoredAny>()?;
                    }
                }
                match type_value.as_deref() {
                    Some("json_object") => Ok(ResponseFormat::JsonObject),
                    _ => Err(de::Error::custom("expected type field with value 'json_object'")),
                }
            }
        }

        deserializer.deserialize_map(ResponseFormatVisitor)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Prediction {
    pub content: String,
}

impl Serialize for Prediction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", "content")?;
        map.serialize_entry("content", &self.content)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Prediction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PredictionVisitor;

        impl<'de> Visitor<'de> for PredictionVisitor {
            type Value = Prediction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with 'type' field set to 'content' and 'content' field")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut type_value: Option<String> = None;
                let mut content_value: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => type_value = Some(map.next_value()?),
                        "content" => content_value = Some(map.next_value()?),
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                if type_value.as_deref() != Some("content") {
                    return Err(de::Error::custom("expected type field with value 'content'"));
                }
                match content_value {
                    Some(content) => Ok(Prediction { content }),
                    None => Err(de::Error::missing_field("content")),
                }
            }
        }

        deserializer.deserialize_map(PredictionVisitor)
    }
}

/// Represents the only allowed value for `route`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[repr(u8)]
pub enum Route {
    #[serde(rename = "fallback")]
    Fallback = 1,
}

/// OpenRouter routes requests to the best available providers for your model.
/// By default, requests are load balanced across the top providers to maximize uptime.
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ProviderPreferences {
    /// Whether to allow backup providers to serve requests
    /// - true: (default) when the primary provider (or your custom providers in "order") is unavailable, use the next best provider.
    /// - false: use only the primary/custom provider, and return the upstream error if it's unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Whether to filter providers to only those that support the parameters you've provided.
    /// If this setting is omitted or set to false, then providers will receive only the parameters they support,
    /// and ignore the rest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    /// Data collection setting. If no available model provider meets the requirement,
    /// your request will return an error.
    /// - allow: (default) allow providers which store user data non-transiently and may train on it
    /// - deny: use only providers which do not collect user data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<DataCollection>,
    /// An ordered list of provider names.
    /// The router will attempt to use the first provider in the subset of this list that supports your requested model,
    /// and fall back to the next if it is unavailable.
    /// If no providers are available, the request will fail with an error message.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub order: Vec<Provider>,
    /// List of provider names to ignore.
    /// If provided, this list is merged with your account-wide ignored provider settings for this request.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ignore: Vec<Provider>,
    /// A list of quantization levels to filter the provider by.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub quantizations: Vec<Quantization>,
    /// The sorting strategy to use for this request, if "order" is not specified.
    /// When set, no load balancing is performed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sorting>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DataCollection {
    #[serde(rename = "allow")]
    #[default]
    Allow = 1,
    #[serde(rename = "deny")]
    Deny,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Quantization {
    #[serde(rename = "int8")]
    Int8 = 1,
    #[serde(rename = "fp6")]
    Fp6,
    #[serde(rename = "int4")]
    Int4,
    #[serde(rename = "fp8")]
    Fp8,
    #[serde(rename = "fp16")]
    Fp16,
    #[serde(rename = "bf16")]
    Bf16,
    #[serde(rename = "fp32")]
    Fp32,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Sorting {
    #[serde(rename = "price")]
    Price = 1,
    #[serde(rename = "throughput")]
    Throughput,
    #[serde(rename = "latency")]
    Latency,
}

// type Message =
//   | {
//       role: 'user' | 'assistant' | 'system';
//       // ContentParts are only for the "user" role:
//       content: string | ContentPart[];
//       // If "name" is included, it will be prepended like this
//       // for non-OpenAI models: `{name}: {content}`
//       name?: string;
//     }
//   | {
//       role: 'tool';
//       content: string;
//       tool_call_id: string;
//       name?: string;
//     };
/// Represents a Message which can be one of:
/// - A user/assistant/system message with content as either a plain string or an array of ContentPart.
/// - A tool message with additional tool_call_id.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum Message {
    User {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Assistant {
        #[serde(skip_serializing_if = "skip_content_if_empty")]
        content: Option<Content>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    System {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Tool {
        content: String,
        tool_call_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

fn skip_content_if_empty(content: &Option<Content>) -> bool {
    match content {
        None => true,
        Some(c) => match c {
            Content::Plain(c) => c.is_empty(),
            Content::Parts(p) => {
                if p.is_empty() {
                    return true;
                }

                p.iter().all(|c| match c {
                    ContentPart::Text { text } => text.is_empty(),
                    ContentPart::ImageUrl { image_url } => {
                        image_url.url.is_empty()
                            && image_url.detail.as_ref().is_none_or(|d| d.is_empty())
                    }
                    ContentPart::File {
                        filename,
                        file_data,
                    } => filename.is_empty() && file_data.is_empty(),
                })
            }
        },
    }
}

/// The content of a non-tool message, which can be a plain string or an array of content parts.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Content {
    Plain(String),
    Parts(Vec<ContentPart>),
}

// type ContentPart = TextContent | ImageContentPart;
/// Represents a part of content which can be either a text block or an image.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum ContentPart {
    // type TextContent = {
    //   type: 'text';
    //   text: string;
    // };
    #[serde(rename = "text")]
    Text { text: String },
    // type ImageContentPart = {
    //   type: 'image_url';
    //   image_url: {
    //     url: string; // URL or base64 encoded image data
    //     detail?: string; // Optional, defaults to "auto"
    //   };
    // };
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },

    //{
    //    "type": "file",
    //    "file": {
    //        "filename": "document.pdf",
    //        "file_data": data_url
    //    }
    //}
    #[serde(rename = "file")]
    File { filename: String, file_data: String },
}

/// Represents an image content part.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageUrl {
    /// URL or base64 encoded image data.
    pub url: String,
    /// Optional detail. Defaults to "auto". If equal to "auto", it will be omitted during serialization.
    #[serde(skip_serializing_if = "is_auto")]
    pub detail: Option<String>,
}

/// Returns true if the detail equals "auto".
fn is_auto(detail: &Option<String>) -> bool {
    detail.as_ref().map(|d| d == "auto").unwrap_or(true)
}

// type Tool = {
//   type: 'function';
//   function: FunctionDescription;
// };
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tool {
    pub function: FunctionDescription,
}

impl Serialize for Tool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", "function")?;
        map.serialize_entry("function", &self.function)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Tool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ToolVisitor;

        impl<'de> Visitor<'de> for ToolVisitor {
            type Value = Tool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map with 'type' field set to 'function' and 'function' field")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut type_value: Option<String> = None;
                let mut function_value: Option<FunctionDescription> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => type_value = Some(map.next_value()?),
                        "function" => function_value = Some(map.next_value()?),
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                if type_value.as_deref() != Some("function") {
                    return Err(de::Error::custom("expected type field with value 'function'"));
                }
                match function_value {
                    Some(function) => Ok(Tool { function }),
                    None => Err(de::Error::missing_field("function")),
                }
            }
        }

        deserializer.deserialize_map(ToolVisitor)
    }
}

// type FunctionDescription = {
//   description?: string;
//   name: string;
//   parameters: object; // JSON Schema object
// };
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionDescription {
    /// The name of the function.
    pub name: String,

    /// An optional description of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The parameters, represented as a JSON Schema object.
    /// It is recommended to use <https://crates.io/crates/schemars>.
    pub parameters: Value,
}

// type ToolChoice =
//   | 'none'
//   | 'auto'
//   | {
//       type: 'function';
//       function: {
//         name: string;
//       };
//     };
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ToolChoice {
    None = 1,
    Auto,
    Required,
    Function(String),
}

impl Serialize for ToolChoice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ToolChoice::None => serializer.serialize_str("none"),
            ToolChoice::Auto => serializer.serialize_str("auto"),
            ToolChoice::Required => serializer.serialize_str("required"),
            ToolChoice::Function(name) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "function")?;

                #[derive(Serialize)]
                struct FunctionField<'a> {
                    name: &'a String,
                }

                map.serialize_entry("function", &FunctionField { name })?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ToolChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ToolChoiceVisitor;

        impl<'de> Visitor<'de> for ToolChoiceVisitor {
            type Value = ToolChoice;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string ('none', 'auto', 'required') or an object with type 'function'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "none" => Ok(ToolChoice::None),
                    "auto" => Ok(ToolChoice::Auto),
                    "required" => Ok(ToolChoice::Required),
                    _ => Err(de::Error::unknown_variant(value, &["none", "auto", "required"])),
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut type_value: Option<String> = None;
                let mut function_name: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => type_value = Some(map.next_value()?),
                        "function" => {
                            #[derive(Deserialize)]
                            struct FunctionField {
                                name: String,
                            }
                            let func: FunctionField = map.next_value()?;
                            function_name = Some(func.name);
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                if type_value.as_deref() != Some("function") {
                    return Err(de::Error::custom("expected type field with value 'function'"));
                }
                match function_name {
                    Some(name) => Ok(ToolChoice::Function(name)),
                    None => Err(de::Error::missing_field("function")),
                }
            }
        }

        deserializer.deserialize_any(ToolChoiceVisitor)
    }
}

/// As described in the OpenRouter documentation, only [`Reasoning::effort`] or [`Reasoning::max_tokens`]
/// should be set.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Reasoning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Effort {
    #[serde(rename = "high")]
    High = 1,
    #[default]
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Usage {
    pub include: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    r#type: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::completions::request::{
        Content, ContentPart, FunctionDescription, ImageUrl, Message, Prediction, Request,
        ResponseFormat, Route, Stop, Tool, ToolChoice,
    };

    #[test]
    fn route_round_trip() {
        let route = Route::Fallback;
        let serialized = serde_json::to_string(&route).unwrap();
        assert_eq!(serialized, "\"fallback\"");
        let deserialized: Route = serde_json::from_str(&serialized).unwrap();
        assert_eq!(route, deserialized);
    }

    #[test]
    fn effort_round_trip() {
        let effort = Effort::High;
        let serialized = serde_json::to_string(&effort).unwrap();
        assert_eq!(serialized, "\"high\"");
        let deserialized: Effort = serde_json::from_str(&serialized).unwrap();
        assert_eq!(effort, deserialized);

        let effort = Effort::Medium;
        let serialized = serde_json::to_string(&effort).unwrap();
        assert_eq!(serialized, "\"medium\"");
        let deserialized: Effort = serde_json::from_str(&serialized).unwrap();
        assert_eq!(effort, deserialized);

        let effort = Effort::Low;
        let serialized = serde_json::to_string(&effort).unwrap();
        assert_eq!(serialized, "\"low\"");
        let deserialized: Effort = serde_json::from_str(&serialized).unwrap();
        assert_eq!(effort, deserialized);
    }

    #[test]
    fn data_collection_round_trip() {
        let dc = DataCollection::Allow;
        let serialized = serde_json::to_string(&dc).unwrap();
        assert_eq!(serialized, "\"allow\"");
        let deserialized: DataCollection = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dc, deserialized);

        let dc = DataCollection::Deny;
        let serialized = serde_json::to_string(&dc).unwrap();
        assert_eq!(serialized, "\"deny\"");
        let deserialized: DataCollection = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dc, deserialized);
    }

    #[test]
    fn quantization_round_trip() {
        let q = Quantization::Int8;
        let serialized = serde_json::to_string(&q).unwrap();
        assert_eq!(serialized, "\"int8\"");
        let deserialized: Quantization = serde_json::from_str(&serialized).unwrap();
        assert_eq!(q, deserialized);

        let q = Quantization::Fp16;
        let serialized = serde_json::to_string(&q).unwrap();
        assert_eq!(serialized, "\"fp16\"");
        let deserialized: Quantization = serde_json::from_str(&serialized).unwrap();
        assert_eq!(q, deserialized);

        let q = Quantization::Unknown;
        let serialized = serde_json::to_string(&q).unwrap();
        assert_eq!(serialized, "\"unknown\"");
        let deserialized: Quantization = serde_json::from_str(&serialized).unwrap();
        assert_eq!(q, deserialized);
    }

    #[test]
    fn sorting_round_trip() {
        let s = Sorting::Price;
        let serialized = serde_json::to_string(&s).unwrap();
        assert_eq!(serialized, "\"price\"");
        let deserialized: Sorting = serde_json::from_str(&serialized).unwrap();
        assert_eq!(s, deserialized);

        let s = Sorting::Throughput;
        let serialized = serde_json::to_string(&s).unwrap();
        assert_eq!(serialized, "\"throughput\"");
        let deserialized: Sorting = serde_json::from_str(&serialized).unwrap();
        assert_eq!(s, deserialized);

        let s = Sorting::Latency;
        let serialized = serde_json::to_string(&s).unwrap();
        assert_eq!(serialized, "\"latency\"");
        let deserialized: Sorting = serde_json::from_str(&serialized).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn stop_round_trip() {
        let stop = Stop::Single("STOP".to_string());
        let serialized = serde_json::to_string(&stop).unwrap();
        assert_eq!(serialized, "\"STOP\"");
        let deserialized: Stop = serde_json::from_str(&serialized).unwrap();
        assert_eq!(stop, deserialized);

        let stop = Stop::Multiple(vec!["STOP1".to_string(), "STOP2".to_string()]);
        let serialized = serde_json::to_string(&stop).unwrap();
        assert_eq!(serialized, "[\"STOP1\",\"STOP2\"]");
        let deserialized: Stop = serde_json::from_str(&serialized).unwrap();
        assert_eq!(stop, deserialized);
    }

    #[test]
    fn response_format_round_trip() {
        let rf = ResponseFormat::JsonObject;
        let serialized = serde_json::to_string(&rf).unwrap();
        assert_eq!(serialized, "{\"type\":\"json_object\"}");
        let deserialized: ResponseFormat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rf, deserialized);
    }

    #[test]
    fn cache_control_round_trip() {
        let cc = CacheControl {
            r#type: "ephemeral".to_string(),
        };
        let serialized = serde_json::to_string(&cc).unwrap();
        assert_eq!(serialized, "{\"type\":\"ephemeral\"}");
        let deserialized: CacheControl = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cc, deserialized);
    }

    #[test]
    fn usage_round_trip() {
        let usage = Usage { include: true };
        let serialized = serde_json::to_string(&usage).unwrap();
        assert_eq!(serialized, "{\"include\":true}");
        let deserialized: Usage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(usage, deserialized);

        let usage = Usage { include: false };
        let serialized = serde_json::to_string(&usage).unwrap();
        assert_eq!(serialized, "{\"include\":false}");
        let deserialized: Usage = serde_json::from_str(&serialized).unwrap();
        assert_eq!(usage, deserialized);
    }

    #[test]
    fn image_url_round_trip() {
        let img_url = ImageUrl {
            url: "https://example.com/image.png".to_string(),
            detail: Some("high".to_string()),
        };
        let serialized = serde_json::to_string(&img_url).unwrap();
        let deserialized: ImageUrl = serde_json::from_str(&serialized).unwrap();
        assert_eq!(img_url, deserialized);

        let img_url = ImageUrl {
            url: "https://example.com/image.png".to_string(),
            detail: Some("auto".to_string()),
        };
        let serialized = serde_json::to_string(&img_url).unwrap();
        assert_eq!(serialized, "{\"url\":\"https://example.com/image.png\"}");
        let deserialized: ImageUrl = serde_json::from_str(&serialized).unwrap();
        assert_eq!(img_url.url, deserialized.url);

        let img_url = ImageUrl {
            url: "https://example.com/image.png".to_string(),
            detail: None,
        };
        let serialized = serde_json::to_string(&img_url).unwrap();
        assert_eq!(serialized, "{\"url\":\"https://example.com/image.png\"}");
        let deserialized: ImageUrl = serde_json::from_str(&serialized).unwrap();
        assert_eq!(img_url, deserialized);
    }

    #[test]
    fn prediction_round_trip() {
        let pred = Prediction {
            content: "Predicted content".to_string(),
        };
        let serialized = serde_json::to_string(&pred).unwrap();
        assert_eq!(serialized, "{\"type\":\"content\",\"content\":\"Predicted content\"}");
        let deserialized: Prediction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pred, deserialized);
    }

    #[test]
    fn reasoning_round_trip() {
        let reasoning = Reasoning {
            effort: Some(Effort::High),
            max_tokens: None,
            exclude: Some(true),
        };
        let serialized = serde_json::to_string(&reasoning).unwrap();
        let deserialized: Reasoning = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reasoning, deserialized);

        let reasoning = Reasoning {
            effort: None,
            max_tokens: Some(1000),
            exclude: None,
        };
        let serialized = serde_json::to_string(&reasoning).unwrap();
        let deserialized: Reasoning = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reasoning, deserialized);

        let reasoning = Reasoning::default();
        let serialized = serde_json::to_string(&reasoning).unwrap();
        assert_eq!(serialized, "{}");
        let deserialized: Reasoning = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reasoning, deserialized);
    }

    #[test]
    fn content_part_round_trip() {
        let cp = ContentPart::Text {
            text: "Hello, world!".to_string(),
        };
        let serialized = serde_json::to_string(&cp).unwrap();
        let deserialized: ContentPart = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cp, deserialized);

        let cp = ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("high".to_string()),
            },
        };
        let serialized = serde_json::to_string(&cp).unwrap();
        let deserialized: ContentPart = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cp, deserialized);

        let cp = ContentPart::File {
            filename: "document.pdf".to_string(),
            file_data: "base64data".to_string(),
        };
        let serialized = serde_json::to_string(&cp).unwrap();
        let deserialized: ContentPart = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cp, deserialized);
    }

    #[test]
    fn content_round_trip() {
        let content = Content::Plain("Hello, world!".to_string());
        let serialized = serde_json::to_string(&content).unwrap();
        assert_eq!(serialized, "\"Hello, world!\"");
        let deserialized: Content = serde_json::from_str(&serialized).unwrap();
        assert_eq!(content, deserialized);

        let content = Content::Parts(vec![
            ContentPart::Text {
                text: "Hello".to_string(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "https://example.com/image.png".to_string(),
                    detail: None,
                },
            },
        ]);
        let serialized = serde_json::to_string(&content).unwrap();
        let deserialized: Content = serde_json::from_str(&serialized).unwrap();
        assert_eq!(content, deserialized);
    }

    #[test]
    fn function_description_round_trip() {
        let fd = FunctionDescription {
            name: "test_func".to_string(),
            description: Some("A test function".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "arg1": { "type": "string" }
                },
                "required": ["arg1"]
            }),
        };
        let serialized = serde_json::to_string(&fd).unwrap();
        let deserialized: FunctionDescription = serde_json::from_str(&serialized).unwrap();
        assert_eq!(fd, deserialized);

        let fd = FunctionDescription {
            name: "test_func2".to_string(),
            description: None,
            parameters: json!({}),
        };
        let serialized = serde_json::to_string(&fd).unwrap();
        let deserialized: FunctionDescription = serde_json::from_str(&serialized).unwrap();
        assert_eq!(fd, deserialized);
    }

    #[test]
    fn tool_round_trip() {
        let tool = Tool {
            function: FunctionDescription {
                name: "my_function".to_string(),
                description: Some("Does something".to_string()),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "param1": { "type": "string" }
                    }
                }),
            },
        };
        let serialized = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn tool_choice_round_trip() {
        let tc = ToolChoice::None;
        let serialized = serde_json::to_string(&tc).unwrap();
        assert_eq!(serialized, "\"none\"");
        let deserialized: ToolChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tc, deserialized);

        let tc = ToolChoice::Auto;
        let serialized = serde_json::to_string(&tc).unwrap();
        assert_eq!(serialized, "\"auto\"");
        let deserialized: ToolChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tc, deserialized);

        let tc = ToolChoice::Required;
        let serialized = serde_json::to_string(&tc).unwrap();
        assert_eq!(serialized, "\"required\"");
        let deserialized: ToolChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tc, deserialized);

        let tc = ToolChoice::Function("my_func".to_string());
        let serialized = serde_json::to_string(&tc).unwrap();
        let deserialized: ToolChoice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(tc, deserialized);
    }

    #[test]
    fn provider_preferences_round_trip() {
        use crate::providers::Provider;

        let pp = ProviderPreferences {
            allow_fallbacks: Some(false),
            require_parameters: Some(true),
            data_collection: Some(DataCollection::Deny),
            order: vec![Provider::OpenAI],
            ignore: vec![Provider::Anthropic],
            quantizations: vec![Quantization::Fp16, Quantization::Fp32],
            sort: Some(Sorting::Price),
        };
        let serialized = serde_json::to_string(&pp).unwrap();
        let deserialized: ProviderPreferences = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pp, deserialized);

        let pp = ProviderPreferences::default();
        let serialized = serde_json::to_string(&pp).unwrap();
        assert_eq!(serialized, "{}");
        let deserialized: ProviderPreferences = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pp, deserialized);
    }

    #[test]
    fn message_round_trip() {
        let msg = Message::User {
            content: Content::Plain("Hello".to_string()),
            name: Some("Alice".to_string()),
            cache_control: None,
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);

        let msg = Message::Assistant {
            content: Some(Content::Plain("Hi there".to_string())),
            name: None,
            tool_calls: None,
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);

        let msg = Message::System {
            content: Content::Plain("System message".to_string()),
            name: None,
            cache_control: Some(CacheControl {
                r#type: "ephemeral".to_string(),
            }),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);

        let msg = Message::Tool {
            content: "Tool response".to_string(),
            tool_call_id: "call123".to_string(),
            name: Some("ToolName".to_string()),
        };
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn request_round_trip() {
        use crate::primatives::{Temperature, FrequencyPenalty};

        let req = Request {
            messages: Some(vec![Message::User {
                content: Content::Plain("Hello".to_string()),
                name: None,
                cache_control: None,
            }]),
            prompt: None,
            model: Some("gpt-4".to_string()),
            response_format: Some(ResponseFormat::JsonObject),
            stop: Some(Stop::Single("STOP".to_string())),
            stream: Some(false),
            max_tokens: Some(100),
            temperature: Some(Temperature::clamp_new(0.7)),
            tools: None,
            tool_choice: Some(ToolChoice::Auto),
            seed: Some(42),
            top_p: Some(TopP::try_from(0.9).unwrap()),
            top_k: Some(50),
            frequency_penalty: Some(FrequencyPenalty::clamp_new(0.5)),
            presence_penalty: None,
            repetition_penalty: None,
            logit_bias: None,
            top_logprobs: None,
            min_p: None,
            top_a: None,
            user: Some("user123".to_string()),
            prediction: None,
            transforms: None,
            models: Some(vec!["model1".to_string(), "model2".to_string()]),
            route: Some(Route::Fallback),
            provider: None,
            reasoning: Some(Reasoning {
                effort: Some(Effort::Medium),
                max_tokens: None,
                exclude: None,
            }),
            usage: None,
        };
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: Request = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);

        let req = Request::default();
        let serialized = serde_json::to_string(&req).unwrap();
        assert_eq!(serialized, "{}");
        let deserialized: Request = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn tool_choice() {
        let tool_choice = ToolChoice::None;
        let expected = json!("none");
        let actual: Value =
            serde_json::from_str(&serde_json::to_string(&tool_choice).unwrap()).unwrap();
        assert_eq!(expected, actual);

        let tool_choice = ToolChoice::Auto;
        let expected = json!("auto");
        let actual: Value =
            serde_json::from_str(&serde_json::to_string(&tool_choice).unwrap()).unwrap();
        assert_eq!(expected, actual);

        let tool_choice = ToolChoice::Function("test".to_string());
        let expected = json!({
            "type": "function",
            "function": { "name": "test" }
        });
        let actual: Value =
            serde_json::from_str(&serde_json::to_string(&tool_choice).unwrap()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn function_description() {
        let fd = FunctionDescription {
            name: "sample_func".to_string(),
            description: Some("A sample function".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "arg1": { "type": "string" },
                    "arg2": { "type": "number" }
                },
                "required": ["arg1"]
            }),
        };

        let serialized = serde_json::to_string(&fd).unwrap();
        let expected = r#"{"name":"sample_func","description":"A sample function","parameters":{"type":"object","properties":{"arg1":{"type":"string"},"arg2":{"type":"number"}},"required":["arg1"]}}"#;
        assert_eq!(expected, serialized);
    }

    #[test]
    fn tool() {
        let function_description = FunctionDescription {
            description: Some("This is a test function".to_string()),
            name: "test_function".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "param1": { "type": "string" },
                    "param2": { "type": "integer" }
                },
                "required": ["param1"]
            }),
        };

        let tool = Tool {
            function: function_description,
        };

        let serialized = serde_json::to_string(&tool).unwrap();

        let expected: Value = json!({
            "type": "function",
            "function": {
                "description": "This is a test function",
                "name": "test_function",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "param1": { "type": "string" },
                        "param2": { "type": "integer" }
                    },
                    "required": ["param1"]
                }
            }
        });

        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, serialized_value);
    }

    #[test]
    fn text_content() {
        let content = ContentPart::Text {
            text: "Hello, world!".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let expected: Value = json!({
            "type": "text",
            "text": "Hello, world!"
        });

        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, serialized_value);
    }

    #[test]
    fn image_content() {
        let content = ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("custom detail".to_string()),
            },
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let expected: Value = json!({
            "type": "image_url",
            "image_url": {
                "url": "https://example.com/image.png",
                "detail": "custom detail"
            }
        });

        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(serialized_value, expected);
    }

    #[test]
    fn image_url_default() {
        let content = ContentPart::ImageUrl {
            image_url: ImageUrl {
                url: "https://example.com/image.png".to_string(),
                detail: Some("auto".into()),
            },
        };

        let serialized = serde_json::to_string(&content).unwrap();
        let expected: Value = json!({
            "type": "image_url",
            "image_url": {
                "url": "https://example.com/image.png"
                // "detail" is omitted because it's the default value ("auto")
            }
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn user_plain() {
        let msg = Message::User {
            content: Content::Plain("Hello, world!".to_string()),
            name: Some("Alice".to_string()),
            cache_control: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "user",
            "content": "Hello, world!",
            "name": "Alice"
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn user_parts() {
        let msg = Message::User {
            content: Content::Parts(vec![
                ContentPart::Text {
                    text: "Hello".to_string(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "https://example.com/image.png".to_string(),
                        detail: Some("custom".to_string()),
                    },
                },
            ]),
            name: None,
            cache_control: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "user",
            "content": [
                { "type": "text", "text": "Hello" },
                { "type": "image_url", "image_url": { "url": "https://example.com/image.png", "detail": "custom" } }
            ]
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn message_assistant() {
        let msg = Message::Assistant {
            content: Some(Content::Plain("How can I help?".to_string())),
            name: None,
            tool_calls: None,
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "assistant",
            "content": "How can I help?"
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn message_system() {
        let msg = Message::System {
            content: Content::Plain("System message".to_string()),
            name: None,
            cache_control: Some(CacheControl {
                r#type: "ephemeral".to_string(),
            }),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "system",
            "content": "System message",
            "cache_control": {
                "type": "ephemeral",
            },
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn message_tool() {
        let msg = Message::Tool {
            content: "Tool response".to_string(),
            tool_call_id: "call123".to_string(),
            name: Some("ToolName".to_string()),
        };

        let serialized = serde_json::to_string(&msg).unwrap();
        let expected: Value = json!({
            "role": "tool",
            "content": "Tool response",
            "tool_call_id": "call123",
            "name": "ToolName"
        });
        let actual: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn request() {
        use crate::primatives::{Temperature, FrequencyPenalty, PresencePenalty, RepetitionPenalty, Bias, MinP, TopA};

        // Prepare a sample logit_bias value.
        let mut logit_bias = BTreeMap::new();
        logit_bias.insert("1".to_string(), Bias::clamp_new(0.5));

        // Build a Request instance with a mix of fields.
        let req = Request {
            messages: None,
            prompt: Some("Hello".to_string()),
            model: Some("gpt-3.5-turbo".to_string()),
            response_format: Some(ResponseFormat::JsonObject),
            stop: Some(Stop::Multiple(vec![
                "STOP1".to_string(),
                "STOP2".to_string(),
            ])),
            stream: Some(true),
            max_tokens: Some(100),
            temperature: Some(Temperature::clamp_new(0.5)),
            tools: None,
            tool_choice: None,
            seed: Some(42),
            top_p: Some(TopP::try_from(0.95).unwrap()),
            top_k: Some(10),
            frequency_penalty: Some(FrequencyPenalty::clamp_new(0.2)),
            presence_penalty: Some(PresencePenalty::clamp_new(0.3)),
            repetition_penalty: Some(RepetitionPenalty::clamp_new(0.8)),
            logit_bias: Some(logit_bias),
            top_logprobs: Some(5),
            min_p: Some(MinP::clamp_new(0.1)),
            top_a: Some(TopA::clamp_new(0.2)),
            user: Some("test".into()),
            prediction: Some(Prediction {
                content: "Predicted content".to_string(),
            }),
            transforms: Some(vec!["transform1".to_string(), "transform2".to_string()]),
            models: Some(vec!["model1".to_string(), "model2".to_string()]),
            route: Some(Route::Fallback),
            provider: None,
            reasoning: Some(Reasoning {
                effort: Some(Effort::High),
                max_tokens: None,
                exclude: Some(true),
            }),
            usage: None,
        };

        let serialized = serde_json::to_string(&req).unwrap();

        let expected: Value = json!({
            "prompt": "Hello",
            "model": "gpt-3.5-turbo",
            "response_format": { "type": "json_object" },
            "stop": ["STOP1", "STOP2"],
            "stream": true,
            "max_tokens": 100,
            "temperature": 0.5,
            "seed": 42,
            "top_p": 0.95,
            "top_k": 10,
            "frequency_penalty": 0.2,
            "presence_penalty": 0.3,
            "repetition_penalty": 0.8,
            "logit_bias": { "1": 0.5 },
            "top_logprobs": 5,
            "min_p": 0.1,
            "top_a": 0.2,
            "user": "test",
            "prediction": { "type": "content", "content": "Predicted content" },
            "transforms": ["transform1", "transform2"],
            "models": ["model1", "model2"],
            "route": "fallback",
            "reasoning": { "effort": "high", "exclude": true }
        });

        let actual: Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expected, actual);
    }
}
