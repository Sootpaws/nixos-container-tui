// Re-export macros to get them in the right place
pub use crate::{log, report_async};

/// Get the systemd service name for a container
pub fn service_name(container: &str) -> String {
    format!("container@{container}.service")
}

/// Helper for running a fallable async function and reporting returned errors
#[macro_export]
macro_rules! report_async {
    ( $( #[ $meta:meta ] )* $vis:vis $name:ident
        [ $container:ident , $channel:ident ]
        ( $( $arg_name:ident : $ty:ty ),* )
        $body:block
        $context:literal
    ) => {
        $( #[ $meta ] )*
        $vis async fn $name (
            container: &'static str,
            channel: Sender,
            $( $arg_name : $ty , )*
        ) {
            let inner = async |
                $container : &'static str,
                $channel : Sender,
                $( $arg_name : $ty , )*
            | -> Result<()> { $body };
            match inner(
                container,
                channel.clone(),
                $( $arg_name , )*
            ).await.context( $context ) {
                Ok(()) => (),
                Err(error) => channel
                    .send(NamedUpdate {
                        container_name: container,
                        inner: Update::Error(error),
                    })
                    .expect("Channel should always be open"),
            }
        }
    }
}

/// Helper for reporting log messages from container monitors
#[macro_export]
macro_rules! log {
    ($name:expr, $sender:expr, $message:expr) => {
        $sender
            .send(NamedUpdate {
                container_name: $name,
                inner: Update::Log(format!($message)),
            })
            .unwrap()
    };
}
