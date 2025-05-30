use anyhow::Result;
use kameo::{
    Actor,
    prelude::{Context, Message},
};
use thiserror::Error;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Error, PartialEq)]
pub enum MyError {
    #[error("Failed: {0}")]
    AnError(u32),
}

pub struct MyActor;

impl Actor for MyActor {
    type Error = kameo::error::Infallible;
}

struct Request(String);

impl Message<Request> for MyActor {
    type Reply = Result<String, MyError>;

    async fn handle(&mut self, msg: Request, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        let _ = ctx.reply(Ok(msg.0.to_uppercase()));
        Ok(msg.0)
    }
}
struct Fail(u32);

impl Message<Fail> for MyActor {
    type Reply = Result<(), MyError>;

    async fn handle(&mut self, msg: Fail, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        Err(MyError::AnError(msg.0))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let actor = kameo::spawn(MyActor);

    info!("Sending first message");
    let res = actor.ask(Request("First".to_string())).await;
    match res {
        Ok(str) => assert_eq!(str, "FIRST".to_string()),
        Err(e) => {
            warn!("{:?}", e);
            panic!("Expected OK {:?}", e)
        }
    }

    info!("Sending sync fail message");
    let res = actor.ask(Fail(0)).await;
    assert!(res.is_err());

    info!("Sending second message");
    let res = actor.ask(Request("Second".to_string())).await;
    match res {
        Ok(str) => assert_eq!(str, "SECOND".to_string()),
        Err(e) => {
            warn!("{:?}", e);
            panic!("Expected OK {:?}", e)
        }
    }

    info!("Sending async fail message");
    let () = actor.tell(Fail(1)).await?;

    info!("Sending third message");
    let res = actor.ask(Request("Third".to_string())).await;
    assert!(res.is_err());

    info!("Done");
    Ok(())
}
