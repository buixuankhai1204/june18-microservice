use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::FutureProducer;
use rdkafka::{ClientConfig, Message};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub server_url: String,
    pub timeout_ms: String,
    pub allow_auto_create_topics: String,
    pub enable_auto_commit: String,
}

impl KafkaConfig {
    pub fn new() -> Self {
        KafkaConfig {
            server_url: "localhost:9092".to_string(),
            timeout_ms: "5000".to_string(),
            allow_auto_create_topics: true.to_string(),
            enable_auto_commit: true.to_string(),
        }
    }
    pub(crate) fn create_kafka_producer(&self) -> FutureProducer {
        let url = self.server_url.to_owned();

        let log_level: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", url)
            .set("message.timeout.ms", self.timeout_ms.to_owned())
            .set("allow.auto.create.topics", self.allow_auto_create_topics.to_owned())
            .create()
            .expect("Producer creation error");

        log_level
    }

    fn create_kafka_consumer(&self) -> StreamConsumer {
        let url = self.server_url.to_owned();

        ClientConfig::new()
            .set("group.id", "traffic")
            .set("bootstrap.servers", url)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", self.timeout_ms.to_owned())
            .set("enable.auto.commit", self.enable_auto_commit.to_owned())
            // only store offset from the consumer
            .set("enable.auto.offset.store", "false")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create()
            .expect("Consumer creation failed")
    }
}

#[tracing::instrument(skip_all)]
pub async fn kafka_consumer_task(con: StreamConsumer, db: DatabaseConnection) {
    con.subscribe(&["channels", "departments"]).expect("Failed to subscribe to topics");

    tracing::info!("Starting the consumer loop...");

    loop {
        match con.recv().await {
            Err(e) => tracing::warn!("Kafka error: {}", e),
            Ok(m) => {
                let Some(payload) = m.payload() else {
                    tracing::error!("Could not find a payload :(");
                    continue;
                };

                // here we use `from_slice()` as we initally send it as &[u8]
                let message: KafkaMessage = match serde_json::from_slice(payload) {
                    Ok(res) => res,
                    Err(e) => {
                        // if there is a deserialization error, print an error
                        // and go to the next loop iteration
                        tracing::error!("Deserialization error: {e}");
                        continue;
                    },
                };

                match message.action {
                    Action::CreateChannel => {
                        // Do something after create
                    },

                    // Action::UpdateProgramFromRescheduleCommandHandler => {
                    //     let data: UpdateProgramCommand =
                    //         serde_json::from_value(message.data).unwrap();
                    //     let detail_frame_service = DetailFrameService::new(None, None);
                    //     let tx = db.begin().await.expect("Failed to begin transaction");
                    //     match detail_frame_service
                    //         .update_detail_frame_from_reschedule_command_handler(
                    //             &tx, message.id, &data,
                    //         )
                    //         .await
                    //     {
                    //         Ok(_) => {
                    //             tx.commit().await.expect("Failed to commit transaction");
                    //             tracing::info!(
                    //                 "Successfully updated traffic from reschedule command handler"
                    //             );
                    //         },
                    //         Err(e) => {
                    //             tx.rollback().await.expect("Failed to rollback transaction");
                    //             tracing::error!("Failed to update traffic: {}", e);
                    //         },
                    //     }
                    // },
                    _ => {},
                }

                // print out our payload

                let _ = con
                    .store_offset_from_message(&m)
                    .inspect_err(|e| tracing::warn!("Error while storing offset: {}", e));
            },
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KafkaMessage {
    pub action: Action,
    pub id: i64,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    CreateChannel,
    UpdateProgramFromRescheduleCommandHandler,
    Update,
    Delete,
}
