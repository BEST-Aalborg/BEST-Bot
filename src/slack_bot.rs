extern crate serde_json;

use template::slack;
use template::slack::{Event, RtmClient, Message};
use template::api::{Channel, conversations, requests, requests::Client};
use template::plugin_api_v1;
use template::plugin_api_v2;

use config::CONFIG;

use plugin_manager::PluginType;
use plugin_manager::PluginApi;

use std::collections::BTreeMap;
use std::thread;
use std::sync::{Arc, RwLock};

enum PluginVersion {
    _1(PluginType<plugin_api_v1::Plugin>),
    _2(PluginType<plugin_api_v2::Plugin>),
}

pub struct MyHandler {
    thread: Option<thread::JoinHandle<()>>,
    receiver: plugin_api_v2::Receiver,
    conversation: Arc<RwLock<BTreeMap<String, Channel>>>,
    message_standard: Vec<PluginVersion>,
}

pub trait MyEventHandler: slack::EventHandler {
    fn new(receiver: plugin_api_v2::Receiver) -> MyHandler;
    fn init(&mut self) -> Result<(), slack::Error>;
    fn subscript_to_v1(&mut self, plugin: &PluginApi<plugin_api_v1::Plugin>);
    fn subscript_to_v2(&mut self, plugin: &PluginApi<plugin_api_v2::Plugin>);
    fn request_handler(&mut self);
    fn conversation_info(&mut self, client: &Client, id: &str);
}

#[allow(unused_variables)]
impl MyEventHandler for MyHandler {
    fn new(receiver: plugin_api_v2::Receiver) -> MyHandler {
        MyHandler {
            thread: None,
            receiver: receiver,
            conversation: Arc::new(RwLock::new(BTreeMap::new())),
            message_standard: Vec::new(),
        }
    }

    /// Login to Slack and start The Slack Bot
    fn init(&mut self) -> Result<(), slack::Error> {
        RtmClient::login_and_run::<MyHandler>(&CONFIG.slack.api_token, self)
    }

    /// Add a reference of the plugin to the different events lists that to plugin subscripted to
    fn subscript_to_v1(&mut self, plugin: &PluginApi<plugin_api_v1::Plugin>) {
                for sub in plugin.plugin.event_subscript() {
            match sub {
                plugin_api_v1::EventSubscribe::StandardMessage => {
                    self.message_standard.push(PluginVersion::_1(plugin.plugin.clone()));
                }
            }
        }
    }

    /// Add a reference of the plugin to the different events lists that to plugin subscripted to
    fn subscript_to_v2(&mut self, plugin: &PluginApi<plugin_api_v2::Plugin>) {
        for sub in plugin.plugin.event_subscript() {
            match sub {
                plugin_api_v2::EventSubscribe::StandardMessage => {
                    self.message_standard.push(PluginVersion::_2(plugin.plugin.clone()));
                }
            }
        }
    }

    fn request_handler(&mut self) {
        use template::plugin_api_v2::{Request, Reply};
        use template::channel_return::ReceiverReturn;


        if self.thread.is_none() {
            let receiver = self.receiver.clone();
            let conversation = self.conversation.clone();

            self.thread = Some(thread::spawn(move || {
                let client = requests::default_client().unwrap();;
                loop {
                    let result = ReceiverReturn::recv(&receiver, |request: Request| {
                        match request {
                            Request::ApiToken => Reply::ApiToken(CONFIG.slack.api_token.clone()),
                            Request::AdminApiToken => Reply::AdminApiToken(CONFIG.slack.admin_api_token.clone()),

                            Request::WebHooksIncomingToken => CONFIG.slack.incoming_webhooks_token.as_ref().map_or(
                                Reply::NotConfigured,
                                |token| Reply::WebHooksIncomingToken(token.clone())
                            ),
                            Request::WebHooksOutgoingToken => CONFIG.slack.outgoing_webhooks_token.as_ref().map_or(
                                Reply::NotConfigured,
                                |token| Reply::WebHooksOutgoingToken(token.clone())
                            ),

                            Request::GetChannelName(id) => {
                                match conversation.read().unwrap().get(&id) {
                                    Some(c) => Reply::ChannelName(c.name.clone().unwrap_or(String::new())),
                                    None => {
                                        let result = conversations::info(&client, &CONFIG.slack.api_token, &conversations::InfoRequest {
                                            channel: &id,
                                            include_locale: None,
                                        }).unwrap();
                                        let r = Reply::ChannelName(result.channel.as_ref().unwrap().name.clone().unwrap_or(String::new()));
                                        conversation.write().unwrap().insert(id.to_string(), result.channel.unwrap());
                                        r
                                    },
                                }
                            },

                            Request::ConfigPath => Reply::ConfigPath(CONFIG.plugin_config_path()),
                        }
                    });
                    if result.is_err() {
                        break;
                    }
                }
            }));
        }
    }

    fn conversation_info(&mut self, client: &Client, id: &str) {
        let result = conversations::info(client, &CONFIG.slack.api_token, &conversations::InfoRequest {
            channel: id,
            include_locale: None,
        });

        self.conversation.write().unwrap().insert(id.to_string(), result.unwrap().channel.unwrap());
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
                        
                        for version in &self.message_standard {
                            match version {
                                &PluginVersion::_1(ref plugin) => plugin.event(plugin_api_v1::Event::StandardMessage(&message)),
                                &PluginVersion::_2(ref plugin) => plugin.event(plugin_api_v2::Event::StandardMessage(&message)),
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

    fn on_connect(&mut self, rtm_client: &RtmClient) {
        let client = requests::default_client().unwrap();

        info!("on_connect");
        // find the general channel id from the `StartResponse`
        let general_channel_id = rtm_client.start_response();

        match &general_channel_id.channels {
            &Some(ref channels) => {
                debug!("--- Channels ---");
                for channel in channels {
                    self.conversation_info(&client, channel.id.as_ref().expect("You have encounter a channel without a id, THIS SHOULD BE IMPOSSIBLE!!!"));
                }
            }
            &None => error!("There are no channels!!!")
        }
        match &general_channel_id.groups {
            &Some(ref groups) => {
                debug!("--- Groups ---");
                for group in groups {
                    self.conversation_info(&client, group.id.as_ref().expect("You have encounter a channel without a id, THIS SHOULD BE IMPOSSIBLE!!!"));
                }
            },
            &None => info!("There are no groups")
        }

//        self.request_handler();
    }
}