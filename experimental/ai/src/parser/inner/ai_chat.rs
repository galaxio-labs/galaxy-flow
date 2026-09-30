use super::super::prelude::*;
use super::call::action_call_args;

use crate::ability::ai::ai_chat::AiChatExecutor as GxAIChat;
use crate::parser::domain::gal_keyword;
use crate::util::OptionFrom;

pub fn gal_ai_chat(input: &mut &str) -> Result<GxAIChat> {
    let mut chat = GxAIChat::default();
    gal_keyword("gx.ai_chat", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "default" || key == "prompt" {
            chat.set_prompt_msg(one.1.to_opt());
        } else if key == "prompt_file" {
            chat.set_prompt_file(one.1.to_opt());
        } else if key == "role" {
            chat.set_role(one.1.to_opt());
        }
    }
    Ok(chat)
}

#[cfg(test)]
mod tests {

    use orion_error::dev::testing::TestAssert;

    use super::*;

    #[test]
    fn ai_chat_msg() {
        let mut data = r#"
             gx.ai_chat( prompt: "1+1=?" ) ;"#;
        let obj = gal_ai_chat(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.prompt_msg(), &Some("1+1=?".to_string()));
    }
    #[test]
    fn ai_chat_default() {
        let mut data = r#"
             gx.ai_chat( "1+1=?" ) ;"#;
        let obj = gal_ai_chat(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.prompt_msg(), &Some("1+1=?".to_string()));
    }
    #[test]
    fn ai_chat_file() {
        let mut data = r#"
             gx.ai_chat( prompt_file: "./ai_chat.txt" , role: "developer") ;"#;
        let obj = gal_ai_chat(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.prompt_file(), &Some("./ai_chat.txt".to_string()));
        assert_eq!(obj.role(), &Some("developer".to_string()));
    }
}
