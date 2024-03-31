use protocol::broker_server::generate::mqtt::mqtt_broker_service_server::MqttBrokerService;

pub struct GrpcBrokerServices {}

impl GrpcBrokerServices {
    pub fn new() -> Self {
        return GrpcBrokerServices {};
    }
}

#[tonic::async_trait]

impl MqttBrokerService for GrpcBrokerServices {}
