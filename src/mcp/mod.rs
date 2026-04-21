// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use anyhow::Result;
use rmcp::handler::server::router::Router;
use rmcp::ServerHandler;

pub struct McpService;

impl ServerHandler for McpService {}

pub struct McpServer {
    _inner: Router<McpService>,
}

impl McpServer {
    pub fn new() -> Result<Self> {
        let server = Router::new(McpService);
        
        Ok(McpServer {
            _inner: server,
        })
    }
}
