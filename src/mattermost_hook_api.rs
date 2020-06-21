#![allow(clippy::option_option)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Incoming webhooks let you POST some data to a Mattermost endpoint to create a message in a channel.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Message {
    /// [Markdown-formatted][] message to display in the post.
    ///
    /// To trigger notifications, use `@<username>`, `@channel` and `@here` like you would in normal Mattermost messaging.
    ///
    /// The value is **mandatory**, if [`attachments`][Message::attachments] is empty.
    ///
    /// [Markdown-formatted]: https://docs.mattermost.com/help/messaging/formatting-text.html
    pub text: Option<String>,
    /// Overrides the channel the message posts in.
    ///
    /// Use the channel's name and not the display name, e.g. use `town-square`, not `Town Square`.
    /// Use an "@" followed by a username to send to a direct message.
    /// Defaults to the channel set during webhook creation.
    /// The webhook can post to any public channel and private channel the webhook creator is in.
    /// Posts to direct messages will appear in the DM between the targeted user and the webhook creator.
    pub channel: Option<String>,
    /// Overrides the username the message posts as.
    ///
    /// Defaults to the username set during webhook creation or the webhook creator's username if the former was not set.
    /// Must be enabled [in the configuration][Mattermost-username].
    ///
    /// [Mattermost-username]: https://docs.mattermost.com/administration/config-settings.html#enable-integrations-to-override-usernames
    pub username: Option<String>,
    /// Overrides the profile picture the message posts with.
    ///
    /// Defaults to the URL set during webhook creation or the webhook creator's profile picture if the former was not set.
    /// Must be enabled [in the configuration][Mattermost-icon].
    ///
    /// [Mattermost-icon]: https://docs.mattermost.com/administration/config-settings.html#enable-integrations-to-override-profile-picture-icons
    pub icon_url: Option<String>,
    /// Overrides the profile picture and icon_url parameter.
    ///
    /// Defaults to none and is not set during webhook creation.
    /// Must be enabled [in the configuration][Mattermost-icon].
    /// The expected content is an emoji name, as typed in a message but without `:`.
    ///
    /// [Mattermost-icon]: https://docs.mattermost.com/administration/config-settings.html#enable-integrations-to-override-profile-picture-icons
    pub icon_emoji: Option<String>,
    /// [Message attachments] used for richer formatting options.
    ///
    /// This value is **mandatory**, if [`text`][Message::text] is not set.
    ///
    /// [Message attachments]: https://docs.mattermost.com/developer/message-attachments.html
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    /// Sets the post `type`, mainly for use by plugins.
    ///
    /// If not blank, must begin with `custom_`.
    pub r#type: Option<String>,
    /// Sets the post props, a JSON property bag for storing extra or meta data on the post.
    ///
    /// See the documentation for [Props] for details.
    pub props: Option<Props>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionEvent {
    /// ID of the user clicking the button
    pub user_id: String,
    /// ID of the post containing the button
    pub post_id: String,
    /// ID of the channel containing the post
    pub channel_id: String,
    /// ID of the team containing the channel
    pub team_id: String,
    /// Webhook provided context for the action, see [`Integration::context`].
    pub context: Value,
}

/// For more details see the [Mattermost documentation](https://docs.mattermost.com/developer/message-attachments.html).
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attachment {
    /// A required plain-text summary of the attachment.
    ///
    /// This is used in notifications, and in clients that don’t support formatted text (e.g. IRC).
    pub fallback: String,
    /// A hex color code that will be used as the left border color for the attachment.
    ///
    /// If not specified, it will default to match the left hand sidebar header background color.
    pub color: Option<String>,
    /// An optional line of text that will be shown above the attachment.
    pub pretext: Option<String>,
    /// The text to be included in the attachment.
    ///
    /// It can be formatted using [Markdown][].
    /// For long texts, the message is collapsed and a "Show More" link is added to expand the message.
    ///
    /// [Markdown]: https://docs.mattermost.com/help/messaging/formatting-text.html
    pub text: Option<String>,
    /// An optional name used to identify the author.
    ///
    /// It will be included in a small section at the top of the attachment.
    pub author_name: Option<String>,
    /// An optional URL used to hyperlink the [`author_name`].
    ///
    /// If no [`author_name`] is specified, this field does nothing.
    ///
    /// [`author_name`]: Attachment::author_name
    pub author_link: Option<String>,
    /// An optional URL used to display a 16x16 pixel icon beside the [`author_name`][Attachment::author_name].
    pub author_icon: Option<String>,
    /// An optional title displayed below the author information in the attachment.
    pub title: Option<String>,
    /// An optional URL used to hyperlink the [`title`].
    ///
    /// If no [`title`] is specified, this field does nothing.
    ///
    /// [`title`]: Attachment::title
    pub title_link: Option<String>,
    /// Fields can be included as an optional array within `attachments`, and are used to display information in a table format inside the attachment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<Field>,
    /// An optional URL to an image file (GIF, JPEG, PNG, BMP, or SVG) that is displayed inside a message attachment.
    ///
    /// Large images are resized to a maximum width of 400px or a maximum height of 300px, while still maintaining the original aspect ratio.
    pub image_url: Option<String>,
    /// An optional URL to an image file (GIF, JPEG, PNG, BMP, or SVG) that is displayed as a 75x75 pixel thumbnail on the right side of an attachment.
    ///
    /// We recommend using an image that is already 75x75 pixels, but larger images will be scaled down with the aspect ratio maintained.
    pub thumb_url: Option<String>,
    /// An optional line of text that will be displayed at the bottom of the attachment.
    ///
    /// Footers with more than 300 characters will be truncated with an ellipsis (`…`).
    pub footer: Option<String>,
    /// An optional URL to an image file (GIF, JPEG, PNG, BMP, or SVG) that is displayed as a 16x16 pixel thumbnail before the footer text.
    pub footer_icon: Option<String>,
    /// Actions make webhook messages interactive
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<Action>,
}

/// Fields can be included as an optional array within [`attachments`][Attachment], and are used to display information in a table format inside the attachment.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field {
    /// A title shown in the table above the [`value`][Field::value].
    ///
    /// As of v5.14 a title will render emojis properly.
    pub title: Option<String>,
    /// The text value of the field.
    ///
    /// It can be formatted using [Markdown][].
    ///
    /// [Markdown]: https://docs.mattermost.com/help/messaging/formatting-text.html
    pub value: Option<String>,
    /// Optionally set to true or false (boolean) to indicate whether the [`value`][Field::value] is short enough to be displayed beside other values.
    pub short: Option<bool>,
}

/// Sets the post props, a JSON property bag for storing extra or meta data on the post.
///
/// Mainly used by other integrations accessing posts through the REST API.
/// The following keys are reserved:
/// * `from_webhook`
/// * `override_username`
/// * `override_icon_url`
/// * `override_icon_emoji`
/// * `webhook_display_name`
/// * `card`
/// * `attachments`
///
/// Props `card` allows for extra information (markdown formatted text) to be sent to Mattermost that will only be displayed in the RHS panel after a user clicks on an "info" icon displayed alongside the post.
/// The info icon cannot be customized and is only rendered visible to the user if there is card data passed into the message.
/// This is only available in v5.14+.
/// There is currently no Mobile support for `card` functionality.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Props {
    /// Props `card` allows for extra information (markdown formatted text) to be sent to Mattermost that will only be displayed in the RHS panel.
    pub card: Option<String>,
    #[serde(flatten)]
    pub extras: HashMap<String, Value>,
}

/// Actions make webhook messages interactive
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// Give your action a descriptive name.
    pub name: String,
    /// Configure how Mattermost connects back to the webhook endpoint
    pub integration: Integration,
}

/// Configure how Mattermost connects back to the webhook endpoint
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Integration {
    /// The actions are backed by an integration that handles HTTP POST requests when users click the message button.
    ///
    /// The URL parameter determines where this action is sent.
    /// The request contains an application/json JSON string.
    /// As of 5.14, relative URLs are accepted, simplifying the workflow when a plugin handles the action.
    pub url: String,
    /// The requests sent to the specified URL contain the user ID, post ID, channel ID, team ID, and any context that was provided in the action definition.
    ///
    /// The post ID can be used to, for example, delete or edit the post after clicking on a message button.
    pub context: Value,
}

/// Responce from a triggered [`Action`]
///
/// There are two things which can be done in response to an action.
/// The post with the [`Action`]s can the updated.
/// The details are on the [`PostUpdate`] struct.
/// An ephemeral message can be send to the user who triggered the [`Action`].
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActionResponse {
    /// Update the existing message and replace parts of it with
    pub update: Option<PostUpdate>,
    /// Send an ephemeral message to the user triggering the action.
    pub ephemeral_text: Option<String>,
}

/// Update the existing Mattermost post
///
/// For details on all of the fields, see [the code][];
///
/// You can update individual fields of the post.
/// However, this only works in an all or nothing faschion, in that partial updates are not possible.
/// Each field can be in one of three states:
/// * `None`: Do not update the field.
/// * `Some(None)`: Clear all properties, except the username and icon of the original message, as well as whether the message was pinned to channel or contained emoji reactions
/// * `Some(Some(T))`: Post will be updated to `T`. Username and icon of the original message, and whether the message was pinned to channel or contained emoji reactions will not be updated
///
/// [Source](https://docs.mattermost.com/developer/interactive-messages.html#how-do-i-manage-properties-of-an-interactive-message)
///
/// [the code]: https://github.com/mattermost/mattermost-server/blob/73ce92400b78480a8119f8cf359d594782e3c03f/model/post.go#L63
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostUpdate {
    #[serde(default, with = "serde_with::rust::double_option")]
    pub message: Option<Option<String>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    pub props: Option<Option<PostProps>>,
}

impl From<Message> for PostUpdate {
    fn from(msg: Message) -> Self {
        let message = msg.text.map(Some);
        let props = if !msg.attachments.is_empty() || msg.props.is_some() {
            let mut pprops = PostProps::default();
            pprops.attachments = msg.attachments;
            if let Some(props) = msg.props {
                pprops.card = props.card;
                pprops.extras = props.extras;
            };
            Some(Some(pprops))
        } else {
            None
        };
        Self { message, props }
    }
}

#[test]
fn test_convert_message_to_update() {
    let mut msg = Message::default();
    msg.text = Some("Hello World".to_string());
    msg.attachments.push({
        let mut att = Attachment::default();
        att.fallback = "Hello Fallback".to_string();
        att
    });

    let pu: PostUpdate = msg.into();
    if let Some(Some(prop)) = pu.props {
        assert_eq!(prop.attachments.len(), 1);
        assert!(prop.card.is_none());
    } else {
        assert!(
            false,
            "PostUpdate does not follow the expected structure. It should have a PostProps value."
        );
    }
    assert_eq!(pu.message.unwrap().unwrap(), "Hello World");
}

/// Additional properties on a post
///
/// The meaning of the fields is not clearly defined.
/// The struct is mostly similar to the [`Props`] struct on the webhook message.
#[serde_with::skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PostProps {
    /// Props `card` allows for extra information (markdown formatted text) to be sent to Mattermost that will only be displayed in the RHS panel.
    ///
    /// For details see [`Props::card`].
    pub card: Option<String>,
    /// [Message attachments] used for richer formatting options.
    ///
    /// For details see [`Message::attachments`].
    ///
    /// [Message attachments]: https://docs.mattermost.com/developer/message-attachments.html
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    /// Extra values not represented above.
    #[serde(flatten, default)]
    pub extras: HashMap<String, Value>,
}

#[test]
fn test_deserialize_parameters() {
    let s = r##"{"attachments": [{"fallback": "fallback", "pretext": "This is the attachment pretext.","text": "This is the attachment text."}]}"##;
    let msg: Message = serde_json::from_str(&s).unwrap();
    dbg!(msg);

    let s = r##"{
            "attachments": [
                {
                    "fallback": "test",
                    "color": "#FF8000",
                    "pretext": "This is optional pretext that shows above the attachment.",
                    "text": "This is the text of the attachment. It should appear just above an image of the Mattermost logo. The left border of the attachment should be colored orange, and below the image it should include additional fields that are formatted in columns. At the top of the attachment, there should be an author name followed by a bolded title. Both the author name and the title should be hyperlinks.",
                    "author_name": "Mattermost",
                    "author_icon": "http://www.mattermost.org/wp-content/uploads/2016/04/icon_WS.png",
                    "author_link": "http://www.mattermost.org/",
                    "title": "Example Attachment",
                    "title_link": "http://docs.mattermost.com/developer/message-attachments.html",
                    "fields": [
                        {
                            "short":false,
                            "title":"Long Field",
                            "value":"Testing with a very long piece of text that will take up the whole width of the table. And then some more text to make it extra long."
                        },
                        {
                            "short":true,
                            "title":"Column One",
                            "value":"Testing"
                        },
                        {
                            "short":true,
                            "title":"Column Two",
                            "value":"Testing"
                        },
                        {
                            "short":false,
                            "title":"Another Field",
                            "value":"Testing"
                        }
                    ],
                    "image_url": "http://www.mattermost.org/wp-content/uploads/2016/03/logoHorizontal_WS.png"
                }
            ]
          }"##;
    let msg: Message = serde_json::from_str(&s).unwrap();
    dbg!(msg);
}
#[test]
fn test_serialize_parameters() {
    let msg = Message {
        props: Some(Props {
            card: Some("This is a String".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let s = serde_json::to_string_pretty(&msg).unwrap();
    eprintln!("{}", s);
}
