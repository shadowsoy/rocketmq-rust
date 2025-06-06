/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::sync::Arc;

use rocketmq_common::common::broker::broker_config::BrokerConfig;
use rocketmq_common::common::server::config::ServerConfig;
use rocketmq_rust::wait_for_signal;
use rocketmq_store::config::message_store_config::MessageStoreConfig;
use tracing::error;
use tracing::info;

use crate::broker_runtime::BrokerRuntime;

pub struct BrokerBootstrap {
    broker_runtime: BrokerRuntime,
}

impl BrokerBootstrap {
    pub async fn boot(mut self) {
        if !self.initialize().await {
            error!("initialize fail");
            return;
        }
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        self.broker_runtime.shutdown_rx = Some(shutdown_rx);

        tokio::join!(self.start(), wait_for_signal_inner(shutdown_tx));
    }

    async fn initialize(&mut self) -> bool {
        self.broker_runtime.initialize().await
    }

    async fn start(&mut self) {
        self.broker_runtime.start().await;
    }
}

async fn wait_for_signal_inner(shutdown_tx: tokio::sync::broadcast::Sender<()>) {
    tokio::select! {
        _ = wait_for_signal() => {
            info!("Broker Received signal, initiating shutdown...");
        }
    }
    // Send shutdown signal to all tasks
    let _ = shutdown_tx.send(());
}

pub struct Builder {
    broker_config: BrokerConfig,
    message_store_config: MessageStoreConfig,
    server_config: ServerConfig,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            broker_config: Default::default(),
            message_store_config: MessageStoreConfig::default(),
            server_config: Default::default(),
        }
    }

    pub fn set_broker_config(mut self, broker_config: BrokerConfig) -> Self {
        self.broker_config = broker_config;
        self
    }

    pub fn set_message_store_config(mut self, message_store_config: MessageStoreConfig) -> Self {
        self.message_store_config = message_store_config;
        self
    }

    pub fn set_server_config(mut self, server_config: ServerConfig) -> Self {
        self.server_config = server_config;
        self
    }

    pub fn build(self) -> BrokerBootstrap {
        BrokerBootstrap {
            broker_runtime: BrokerRuntime::new(
                Arc::new(self.broker_config),
                Arc::new(self.message_store_config),
                Arc::new(self.server_config),
            ),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
