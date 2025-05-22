use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex w-screen">
                // Sidebar
                <div class="flex-none w-56 h-screen bg-gradient-to-b from-purple-400 via-pink-400 to-red-400 text-white">
                    <div class="text-xl p-3 font-bold">{"Users"}</div>
                    {
                        for self.users.iter().map(|u| {
                            html! {
                                <div class="flex m-3 bg-white bg-opacity-20 hover:bg-opacity-30 rounded-lg p-2 transition">
                                    <img class="w-12 h-12 rounded-full border-2 border-white"
                                         src={u.avatar.clone()} alt="avatar"/>
                                    <div class="flex-grow p-3 text-sm">
                                        <div class="font-semibold">{ &u.name }</div>
                                        <div class="text-xs text-white">{"Ready to chat!"}</div>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>

                // Main chat area
                <div class="grow h-screen flex flex-col bg-gradient-to-tr from-green-200 to-blue-200">
                    // Header
                    <div class="w-full h-14 border-b-2 border-green-300 bg-white bg-opacity-70">
                        <div class="text-xl p-3 font-bold text-green-800">{"💬 Chat!"}</div>
                    </div>

                    // Messages
                    <div class="w-full grow overflow-auto border-b-2 border-blue-300 p-4 space-y-4">
                        {
                            for self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html! {
                                    <div class="flex items-start w-3/6 p-4 bg-white rounded-xl shadow-lg">
                                        <img class="w-8 h-8 rounded-full mr-3 border-2 border-blue-400"
                                             src={user.avatar.clone()} alt="avatar"/>
                                        <div>
                                            <div class="text-sm font-bold text-blue-600">{ &m.from }</div>
                                            <div class="text-sm text-gray-700 mt-1">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3 rounded" src={m.message.clone()}/>
                                                } else {
                                                    { &m.message }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            })
                        }
                    </div>

                    // Input
                    <div class="w-full h-14 flex px-3 items-center bg-white bg-opacity-80">
                        <input
                            ref={self.chat_input.clone()}
                            type="text"
                            placeholder="Message"
                            class="block w-full py-2 pl-4 mx-3 bg-green-50 rounded-full outline-none focus:ring-2 focus:ring-green-400"
                        />
                        <button onclick={submit}
                                class="p-3 shadow-lg bg-green-500 hover:bg-green-600 w-10 h-10 rounded-full
                                       flex justify-center items-center text-white transition">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"
                                 class="fill-white w-6 h-6">
                                <path d="M0 0h24v24H0z" fill="none"/>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}