extern crate serde_json;

use template::slack;
use template::slack::{Event, RtmClient, Message};
use template::plugin_api_v1;

use config::DEFAULT;
use std::collections::BTreeMap;

use plugin_manager::PluginTypeV1;
use plugin_manager::PluginApiV1;
use std::rc::Rc;

enum PluginVersion<T> {
    _1(T),
}

pub struct MyHandler {
    channels: BTreeMap<String, String>,
    message_standard: Vec<PluginTypeV1>,
    message_standard_test: Vec<PluginVersion<PluginTypeV1>>,
}

pub trait MyEventHandler: slack::EventHandler {
    fn new() -> MyHandler;
    fn init(&mut self) -> Result<(), slack::Error>;
    fn subscript_to(&mut self, plugin: &PluginApiV1);
}

#[allow(unused_variables)]
impl MyEventHandler for MyHandler {
    fn new() -> MyHandler {
        MyHandler {
            channels: BTreeMap::new(),
            message_standard: Vec::new(),
            message_standard_test: Vec::new(),
        }
    }

    /// Login to Slack and start The Slack Bot
    fn init(&mut self) -> Result<(), slack::Error> {
        RtmClient::login_and_run::<MyHandler>(&DEFAULT.slack.api_token, self)
    }

    /// Add a reference of the plugin to the different events lists that to plugin subscripted to 
    fn subscript_to(&mut self, plugin: &PluginApiV1) {
        for sub in plugin.plugin.event_subscript() {
            match sub {
                plugin_api_v1::EventSubscribe::StandardMessage => {
                    self.message_standard.push(Rc::clone(&plugin.plugin));
                    self.message_standard_test.push(PluginVersion::_1(Rc::clone(&plugin.plugin)));
                }
            }
        }
    }
}

#[allow(unused_variables)]
impl slack::EventHandler for MyHandler {
    fn on_event(&mut self, client: &RtmClient, event: Event) {
        debug!("on_event(event: {:?})", event);
        match event {
            Event::Message(message) => {
                let message = *message;
                match message {
                    Message::Standard(message) => {
                        debug!("Message::Standard - {:?}", &message);
                        
                        for version in &self.message_standard_test {
                            match version {
                                &PluginVersion::_1(ref plugin) => plugin.event(plugin_api_v1::Event::StandardMessage(&message)),
                            }
                        }
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    fn on_close(&mut self, client: &RtmClient) {
        info!("on_close");
    }

    fn on_connect(&mut self, client: &RtmClient) {

        info!("on_connect");
        // find the general channel id from the `StartResponse`
        let general_channel_id = client.start_response();

        match &general_channel_id.channels {
            &Some(ref channels) => {
                debug!("--- Channels ---");
                for channel in channels {
                    let id = channel.id.clone();
                    let name = channel.name.clone();
                    debug!("{} - {}",
                    id.as_ref().expect("This channel does not have a id!!!"),
                    name.as_ref().expect("This channel does not have a name!!!")
                    );
                    self.channels.insert(id.expect("You have encounter a channel without a id, THIS SHOULD BE IMPOSSIBLE!!!"), name.unwrap());
                }
            },
            &None => error!("There are no channels!!!")
        }
        match &general_channel_id.groups {
            &Some(ref groups) => {
                debug!("--- Groups ---");
                for group in groups {
                    let id = group.id.clone();
                    let name = group.name.clone();
                    debug!("{} - {}",
                             id.as_ref().expect("This group does not have a name!!!"),
                             name.as_ref().expect("This group does not have a name!!!")
                    );
                    self.channels.insert(id.expect("You have encounter a channel without a id, THIS SHOULD BE IMPOSSIBLE!!!"), name.unwrap());
                }
            },
            &None => error!("There are no groups!!!")
        }

        // Send a message over the real time api websocket
    }
}
