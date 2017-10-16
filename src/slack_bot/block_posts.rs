use super::slack::RtmClient;

use super::api::MessageStandard;
use super::api::{chat,im,groups,channels};
use super::api::requests;

use super::serde_json::{ser,Value};
use config::DEFAULT;

pub fn plugin(client: &RtmClient, message: &MessageStandard) {
    let client = requests::default_client().unwrap();

    if message.thread_ts.is_none() {
        let result = chat::delete(&client, &DEFAULT.slack.admin_api_token, &chat::DeleteRequest {
            ts: &message.ts.as_ref().unwrap(),
            channel: &message.channel.as_ref().unwrap(),
            as_user: Some(true),
        });
        println!("DeleteResult: {:?}", result);
        println!("Deleted message form user '{}' in channel '{}'", message.user.as_ref().unwrap(), message.channel.as_ref().unwrap());

        let mut channel_id = String::new();
        let mut channel_name = String::new();
        let result = channels::info(&client, &DEFAULT.slack.api_token, &channels::InfoRequest {
            channel: &message.channel.as_ref().unwrap(),
        });
        match result {
            Ok(c) => {
                channel_id = c.channel.as_ref().unwrap().id.as_ref().unwrap().clone();
                channel_name = c.channel.as_ref().unwrap().name.as_ref().unwrap().clone();
            },
            Err(_) => (),
        }

        let result = groups::info(&client, &DEFAULT.slack.api_token, &groups::InfoRequest {
            channel: &message.channel.as_ref().unwrap(),
        });
        match result {
            Ok(c) => {
                channel_id = c.group.as_ref().unwrap().id.as_ref().unwrap().clone();
                channel_name = c.group.as_ref().unwrap().name.as_ref().unwrap().clone();
            },
            Err(_) => (),
        }

        let result = im::open(&client, &DEFAULT.slack.api_token, &im::OpenRequest {
            user: message.user.as_ref().unwrap(),
            return_im: Some(true),
        });
        let instant_message = result.unwrap().channel.unwrap();

        let result = chat::post_message(&client, &DEFAULT.slack.api_token, &chat::PostMessageRequest {
            channel: instant_message.id.as_ref().unwrap(),
            text: &format!("You are not allowed to create a post this channel <#{}|{}>, please use the thread function. I have attached the message you was trying to post, Stay PANDA!",
                           &channel_id,
                           &channel_name),
            parse: None,
            link_names: None,
            attachments: Some(&format!(r#"[{{"title": "You message", "text": {}}}]"#, &ser::to_string(&Value::String(message.text.as_ref().unwrap().clone())).unwrap())),
            unfurl_links: None,
            unfurl_media: None,
            username: Some("best-bot"),
            as_user: Some(true),
            icon_url: None,
            icon_emoji: Some("best"),
            thread_ts: None,
            reply_broadcast: None,
        });
        println!("PostResult: {:?}", result);
    }
}