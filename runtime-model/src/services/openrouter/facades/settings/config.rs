use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use openrouter::{
    completions::request::{Request, Stop, Reasoning, ProviderPreferences},
	primatives::{Temperature, TopP, TopA, MinP,FrequencyPenalty, PresencePenalty, RepetitionPenalty},
};
use config_crap::boolean::{Boolean};

use crate::adapters::{
    path_helper::{GetTreePath,ServiceReqs,IntoServiceConfig,ServiceConfig},
    service_tree::{get_tree},
};

use super::service_impl::{load_client};

#[derive(Serialize,Deserialize,Clone,PartialEq,Debug)]
pub struct OpenRouterSettingsConfig {
	pub path: String,
	pub open_router_path: String,
    #[serde(default)]
	pub defaults: DefaultsConfig,
}
impl IntoServiceConfig for OpenRouterSettingsConfig {
    fn into_service_config(&self) -> ServiceConfig {
        ServiceConfig::new(self.clone())
    }
}
impl ServiceReqs for OpenRouterSettingsConfig {
    fn creates<'a>(&'a self) -> anyhow::Result<Vec<&'a str>> {
        self.path.get_tree_path()
    }
    fn requires<'a>(&'a self) -> anyhow::Result<Vec<Vec<&'a str>>> {
        Ok(vec![self.open_router_path.get_tree_path()?])
    }
    fn insert_to_tree(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=anyhow::Result<()>> + Send + 'static>> {
        let s = self.clone();
        Box::pin(async move {
            let path = s.path.clone();
            let tree = get_tree();
            if tree.contains_path(&path)? {
                tree.reload(&path, Box::new(s.path.clone())).await?;
            } else {
                load_client(tree,s)?;
            }
            Ok(())
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelDefaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<Temperature>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<TopP>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<FrequencyPenalty>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<PresencePenalty>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<RepetitionPenalty>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_p: Option<MinP>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_a: Option<TopA>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DefaultsConfig {
	#[serde(rename = "override")]
	pub force: Option<Boolean>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<ModelDefaults>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub models: HashMap<String, ModelDefaults>,
}
impl DefaultsConfig {
    fn force(&self) -> bool {
        self.force.as_ref().map(|b| b.as_bool()).unwrap_or(false)
    }

    pub fn update_request(&self, request: &mut Request) {

        // apply a default model first
        if request.model.is_none() && self.default_model.is_some() {
            request.model.clone_from(&self.default_model);
        }

        // alook for correct defaults
        let defaults: &ModelDefaults = {
            let global = self.global.as_ref();

            // first see if we have a model named and fall back to defaults
            let maybe_defaults = match request.model.as_ref().map(|s| s.as_str()) {
                Some(m) => self.models.get(m).into_iter().chain(global).next(),
                None => global,
            };
            match maybe_defaults {
                None => return,
                Some(x) => x,
            }
        };
        if self.force() {
            override_values(request, defaults);
        } else {
            apply_defaults(request, defaults);
        }
    }
}

macro_rules! walk {
	(@OVERRIDE $request: ident, $default: ident; $($field:ident),* $(,)*) => {
		$(
			$request.$field.clone_from(&$default.$field);
		)*
	};
	($request: ident, $default: ident; $($field:ident),* $(,)*) => {
		$(
			if $request.$field.is_none() {
				$request.$field.clone_from(&$default.$field);
			}	
		)*
	};
}

pub fn apply_defaults(request: &mut Request, default: &ModelDefaults) {
	walk!( request, default;
		temperature,
        top_p,
        top_k,
        frequency_penalty,
        presence_penalty,
        repetition_penalty,
        min_p,
        top_a,
        max_tokens,
        seed,
        stop,
        reasoning,
        provider,
        prompt,
	);
}

pub fn override_values(request: &mut Request, default: &ModelDefaults) {
	walk!(@OVERRIDE request, default;
		temperature,
        top_p,
        top_k,
        frequency_penalty,
        presence_penalty,
        repetition_penalty,
        min_p,
        top_a,
        max_tokens,
        seed,
        stop,
        reasoning,
        provider,
        prompt,
	);
}
