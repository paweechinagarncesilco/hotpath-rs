use eyre::Result;
use hotpath::channels::ChannelLogs;
use hotpath::{FunctionLogsJson, FunctionsJson};

/// Fetches metrics from the hotpath HTTP server
pub(crate) fn fetch_metrics(agent: &ureq::Agent, port: u16) -> Result<FunctionsJson> {
    let url = format!("http://localhost:{}/metrics", port);
    let metrics: FunctionsJson = agent
        .get(&url)
        .call()
        .map_err(|e| eyre::eyre!("HTTP request failed: {}", e))?
        .body_mut()
        .read_json()
        .map_err(|e| eyre::eyre!("JSON deserialization failed: {}", e))?;
    Ok(metrics)
}

/// Fetches channels from the hotpath HTTP server
pub(crate) fn fetch_channels(
    agent: &ureq::Agent,
    port: u16,
) -> Result<hotpath::channels::ChannelsJson> {
    let url = format!("http://localhost:{}/channels", port);
    let channels: hotpath::channels::ChannelsJson = agent
        .get(&url)
        .call()
        .map_err(|e| eyre::eyre!("HTTP request failed: {}", e))?
        .body_mut()
        .read_json()
        .map_err(|e| eyre::eyre!("JSON deserialization failed: {}", e))?;
    Ok(channels)
}

/// Fetches recent logs for a specific function
pub(crate) fn fetch_function_logs(
    agent: &ureq::Agent,
    port: u16,
    function_name: &str,
) -> Result<FunctionLogsJson> {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(function_name.as_bytes());

    let url = format!("http://localhost:{}/functions/{}/logs", port, encoded);
    let function_logs: FunctionLogsJson = agent
        .get(&url)
        .call()
        .map_err(|e| eyre::eyre!("HTTP request failed: {}", e))?
        .body_mut()
        .read_json()
        .map_err(|e| eyre::eyre!("JSON deserialization failed: {}", e))?;
    Ok(function_logs)
}

/// Fetches logs for a specific channel from the HTTP server
pub(crate) fn fetch_channel_logs(
    agent: &ureq::Agent,
    port: u16,
    channel_id: u64,
) -> Result<ChannelLogs> {
    let url = format!("http://localhost:{}/channels/{}/logs", port, channel_id);
    let logs: ChannelLogs = agent
        .get(&url)
        .call()
        .map_err(|e| eyre::eyre!("HTTP request failed: {}", e))?
        .body_mut()
        .read_json()
        .map_err(|e| eyre::eyre!("JSON deserialization failed: {}", e))?;
    Ok(logs)
}
