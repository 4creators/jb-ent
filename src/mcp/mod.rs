// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Jacek Błaszczyński

use mcp_rust_sdk::server::{Server, ServerOptions};
use anyhow::Result;

pub struct McpServer {
    inner: Server,
}

impl McpServer {
    pub fn new() -> Result<Self> {
        let options = ServerOptions {
            name: "jb-ent".to_string(),
            version: "0.1.0".to_string(),
            ..Default::default()
        };
        
        let server = Server::new(options);
        
        // Initial SDK setup
        Ok(McpServer {
            inner: server,
        })
    }
}
