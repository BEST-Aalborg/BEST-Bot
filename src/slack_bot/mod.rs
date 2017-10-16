extern crate slack_api as api;
extern crate slack;
extern crate serde_json;

use config::DEFAULT;
use std::collections::BTreeMap;

use self::slack::{Event, RtmClient, Message};
use self::slack::EventHandler;

use self::api::{Channel,ChannelPurpose, ChannelTopic};

mod block_posts;

pub fn init() {
    let mut handler = MyHandler {
        channels: BTreeMap::new(),
    };
    RtmClient::login_and_run(&DEFAULT.slack.api_token, &mut handler);
}
pub struct MyHandler {
    channels: BTreeMap<String, String>,
}

#[allow(unused_variables)]
impl EventHandler for MyHandler {
    fn on_event(&mut self, client: &RtmClient, event: Event) {
//        println!("on_event(event: {:?})", event);
        if DEFAULT.channels.is_some() {
            match event {
                Event::Message(message) => {
                    let message = *message;
                    match message {
                        Message::Standard(message) => {
                            println!("{:?}", &message);
                            let channel_id = message.channel.as_ref().expect("Message did not have a channel, THIS IS IMPOSSIBLE!!!");
                            if self.channels.get(channel_id).is_none() {
                                update_channels_list();
                            }

                            match self.channels.get(channel_id) {
                                Some(channel) => {
                                    let channel = format!("#{}", channel);

                                    println!("Channel: {}", &channel);
                                    if DEFAULT.channels.as_ref().unwrap().get(&channel).is_some() {
                                        let channel = DEFAULT.channels.as_ref().unwrap().get(&channel).unwrap();
                                        for plugin in &channel.plugins {
                                            match plugin.as_ref() {
                                                "block_posts" => block_posts::plugin(client, &message),
                                                _ => println!("The plugin '{}' is not supported", plugin)
                                            }
                                        }
                                    }
                                },
                                None => ()
                            }
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        } else {
            println!("No channels have been configured")
        }
    }

    fn on_close(&mut self, client: &RtmClient) {
        println!("on_close");
    }

    fn on_connect(&mut self, client: &RtmClient) {

        println!("on_connect");
        // find the general channel id from the `StartResponse`
        let general_channel_id = client.start_response();

        match &general_channel_id.channels {
            &Some(ref channels) => {
                println!("--- Channels ---");
                for channel in channels {
                    let id = channel.id.clone();
                    let name = channel.name.clone();
                    println!("{} - {}",
                    id.as_ref().expect("This channel does not have a id!!!"),
                    name.as_ref().expect("This channel does not have a name!!!")
                    );
                    self.channels.insert(id.expect("You have encounter a channel without a id, THIS SHOULD BE IMPOSSIBLE!!!"), name.unwrap());
                }
            },
            &None => println!("There are no channels!!!")
        }
        match &general_channel_id.groups {
            &Some(ref groups) => {
                println!("--- Groups ---");
                for group in groups {
                    let id = group.id.clone();
                    let name = group.name.clone();
                    println!("{} - {}",
                             id.as_ref().expect("This group does not have a name!!!"),
                             name.as_ref().expect("This group does not have a name!!!")
                    );
                    self.channels.insert(id.expect("You have encounter a channel without a id, THIS SHOULD BE IMPOSSIBLE!!!"), name.unwrap());
                }
            },
            &None => println!("There are no groups!!!")
        }

        // Send a message over the real time api websocket
    }
}

fn update_channels_list() {
    println!("TODO: implement a way to update in app list of channels")
}