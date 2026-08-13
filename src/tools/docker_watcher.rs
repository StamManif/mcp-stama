use crate::tools::McpTool;
use anyhow::Result;
use bollard::container::ListContainersOptions;
use bollard::Docker;
use serde_json::json;
use sysinfo::System;

#[derive(Default)]
pub struct DockerWatcherTool;

impl DockerWatcherTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl McpTool for DockerWatcherTool {
    fn name(&self) -> &'static str {
        "docker_watcher"
    }

    fn description(&self) -> &'static str {
        "Inspects local Docker daemon status, active containers, and system environment metrics (RAM, CPU, OS)."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "include_all_containers": {
                    "type": "boolean",
                    "description": "If true, returns all containers (including stopped ones).",
                    "default": false
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value> {
        let include_all_containers = args
            .get("include_all_containers")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Collect system metrics via sysinfo
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu_all();


        let os_name = System::long_os_version()
            .or_else(|| {
                System::name().map(|n| {
                    format!("{} {}", n, System::os_version().unwrap_or_default())
                })
            })
            .unwrap_or_else(|| "Unknown OS".to_string());

        let cpu_cores = sys.cpus().len();
        let total_ram_mb = sys.total_memory() / (1024 * 1024);
        let used_ram_mb = sys.used_memory() / (1024 * 1024);

        let system_info = json!({
            "os": os_name,
            "cpu_cores": cpu_cores,
            "total_ram_mb": total_ram_mb,
            "used_ram_mb": used_ram_mb
        });

        // Inspect Docker daemon status via bollard
        let docker_info = match Docker::connect_with_local_defaults() {
            Ok(docker) => match docker.version().await {
                Ok(version_info) => {
                    let version_str = version_info
                        .version
                        .unwrap_or_else(|| "unknown".to_string());

                    let options = ListContainersOptions::<String> {
                        all: include_all_containers,
                        ..Default::default()
                    };


                    match docker.list_containers(Some(options)).await {
                        Ok(containers) => {
                            let mut container_list = Vec::new();
                            for c in containers {
                                let id = c.id.unwrap_or_default();
                                let short_id = if id.len() > 12 { &id[..12] } else { &id };
                                let name = c
                                    .names
                                    .and_then(|n| n.first().cloned())
                                    .unwrap_or_else(|| "unnamed".to_string());
                                let image = c.image.unwrap_or_default();
                                let status = c.status.unwrap_or_else(|| {
                                    c.state.unwrap_or_else(|| "unknown".to_string())
                                });

                                let ports_str = c
                                    .ports
                                    .map(|ports| {
                                        ports
                                            .iter()
                                            .map(|p| {
                                                if let Some(host) = p.public_port {
                                                    format!("{}:{}", host, p.private_port)
                                                } else {
                                                    format!("{}", p.private_port)
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .unwrap_or_default();


                                container_list.push(json!({
                                    "id": short_id,
                                    "name": name,
                                    "image": image,
                                    "status": status,
                                    "ports": ports_str
                                }));
                            }

                            json!({
                                "available": true,
                                "version": version_str,
                                "active_containers": container_list
                            })
                        }
                        Err(e) => json!({
                            "available": false,
                            "error": format!("Failed to list containers: {}", e)
                        }),
                    }
                }
                Err(e) => json!({
                    "available": false,
                    "error": format!("Docker daemon is not running or socket is inaccessible: {}", e)
                }),
            },
            Err(e) => json!({
                "available": false,
                "error": format!("Docker daemon is not running or socket is inaccessible: {}", e)
            }),
        };

        Ok(json!({
            "system": system_info,
            "docker": docker_info
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_watcher_schema() {
        let tool = DockerWatcherTool::new();
        assert_eq!(tool.name(), "docker_watcher");
        let schema = tool.schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["include_all_containers"].is_object());
    }

    #[tokio::test]
    async fn test_system_metrics() {
        let tool = DockerWatcherTool::new();
        let res = tool.execute(json!({})).await.unwrap();

        assert!(res["system"]["os"].is_string());
        assert!(res["system"]["cpu_cores"].as_u64().unwrap() > 0);
        assert!(res["system"]["total_ram_mb"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_docker_fallback() {
        let tool = DockerWatcherTool::new();
        let res = tool.execute(json!({})).await.unwrap();

        // Must always return structured system metrics and docker object without panicking
        assert!(res.get("system").is_some());
        assert!(res.get("docker").is_some());
        assert!(res["docker"].get("available").is_some());
    }
}
