use crate::{
    dispatching::{tel, updates, DispatcherBuilder, UpdateWithCx},
    dummies::text_message,
    types::{Message, Update, UpdateKind},
    Bot,
};
use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[tokio::test]
async fn test() {
    let handled = Arc::new(AtomicBool::new(false));
    let handled2 = handled.clone();

    let dispatcher = DispatcherBuilder::<Infallible, _>::new(dummy_bot(), "bot_name")
        .handle(updates::message().common().by(move |message: Message| {
            assert_eq!(message.text().unwrap(), "text");
            handled2.store(true, Ordering::SeqCst);
        }))
        .handle(updates::callback_query().by(move || unreachable!()))
        .error_handler(|_| async { unreachable!() })
        .build();

    let message = Update::new(0, UpdateKind::Message(text_message("text")));

    dispatcher.dispatch_one(message).await;

    assert!(handled.load(Ordering::SeqCst));
}

#[tokio::test]
async fn or_else() {
    let in_or_else = Arc::new(AtomicBool::new(false));

    let dispatcher = DispatcherBuilder::<Infallible, _>::new(dummy_bot(), "bot_name")
        .handle(
            updates::message()
                .common()
                .with_text(|text: &str| text == "text")
                .or_else({
                    let in_or_else = in_or_else.clone();
                    move || {
                        in_or_else.store(true, Ordering::SeqCst);
                    }
                })
                .by(|| unreachable!()),
        )
        .error_handler(|_| async { unreachable!() })
        .build();

    let message = Update::new(0, UpdateKind::Message(text_message("not_text")));

    dispatcher.dispatch_one(message).await;

    assert!(in_or_else.load(Ordering::SeqCst));
}

#[tokio::test]
async fn or() {
    let handled = Arc::new(AtomicBool::new(false));

    let dispatcher = DispatcherBuilder::<Infallible, _>::new(dummy_bot(), "bot_name")
        .handle(
            updates::message()
                .common()
                .with_text(|text: &str| text == "text")
                .or_with_text(|text: &str| text == "text2")
                .by({
                    let handled = handled.clone();
                    move || handled.store(true, Ordering::SeqCst)
                }),
        )
        .error_handler(|_| async { unreachable!() })
        .build();

    let message = Update::new(0, UpdateKind::Message(text_message("text2")));

    dispatcher.dispatch_one(message).await;

    assert!(handled.load(Ordering::SeqCst));
}

#[tokio::test]
async fn async_guards() {
    let dispatcher = DispatcherBuilder::<Infallible, _>::new(dummy_bot(), "bot_name")
        .handle(
            updates::message()
                .common()
                .with_chat_id(|id: &i64| {
                    let id = id.clone();
                    async move { id == 10 }
                })
                .by(|mes: Message| assert_eq!(mes.chat.id, 10)),
        )
        .error_handler(|_| async {})
        .build();

    let message = Update::new(0, UpdateKind::Message(text_message("text2")));

    dispatcher.dispatch_one(message).await;
}

#[tokio::test]
async fn update_with_cx() {
    let dispatcher = DispatcherBuilder::<Infallible, _>::new(dummy_bot(), "bot_name")
        .handle(
            updates::message()
                .common()
                .by(|cx: UpdateWithCx<Message>| assert_eq!(cx.update.text().unwrap(), "text2")),
        )
        .error_handler(|_| async {})
        .build();

    let message = Update::new(0, UpdateKind::Message(text_message("text2")));

    dispatcher.dispatch_one(message).await;
}

fn dummy_bot() -> Bot {
    Bot::builder().token("").build()
}