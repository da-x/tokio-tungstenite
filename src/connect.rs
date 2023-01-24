//! Connection helper.
use std::path::Path;

use tokio::net::{TcpStream, UnixStream};

use tungstenite::{
    error::{Error, UrlError},
    handshake::client::Response,
    protocol::WebSocketConfig,
};

use crate::{domain, stream::MaybeTlsStream, IntoClientRequest, WebSocketStream};

#[cfg(any(feature = "native-tls", feature = "__rustls-tls"))]
use crate::Connector;

/// Connect to a given URL.
pub async fn connect_async<R>(
    request: R,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_async_with_config(request, None, false).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration.
/// Please refer to `connect_async()` for more details. `disable_nagle` specifies if
/// the Nagle's algorithm must be disabled, i.e. `set_nodelay(true)`. If you don't know
/// what the Nagle's algorithm is, better leave it set to `false`.
pub async fn connect_async_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request = request.into_client_request()?;

    let domain = domain(&request)?;
    let port = request
        .uri()
        .port_u16()
        .or_else(|| match request.uri().scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(Error::Url(UrlError::UnsupportedUrlScheme))?;

    let addr = format!("{}:{}", domain, port);
    let try_socket = TcpStream::connect(addr).await;
    let socket = try_socket.map_err(Error::Io)?;

    if disable_nagle {
        socket.set_nodelay(true)?;
    }

    crate::tls::client_async_tls_with_config(request, socket, config, None).await
}

/// Connect to a given URL but connect to UNIX domain socket
#[cfg(feature = "uds")]
pub async fn connect_unix_async<R>(
    path: impl AsRef<Path>,
    request: R,
) -> Result<(WebSocketStream<MaybeTlsStream<UnixStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    connect_unix_async_with_config(path, request, None).await
}

/// The same as `connect_unix_async()` but the one can specify a websocket configuration.
#[cfg(feature = "uds")]
pub async fn connect_unix_async_with_config<R>(
    path: impl AsRef<Path>,
    request: R,
    config: Option<WebSocketConfig>,
) -> Result<(WebSocketStream<MaybeTlsStream<UnixStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request = request.into_client_request()?;
    let try_socket = UnixStream::connect(path).await;
    let socket = try_socket.map_err(Error::Io)?;

    crate::tls::client_async_tls_with_config(request, socket, config, None).await
}

/// The same as `connect_async()` but the one can specify a websocket configuration,
/// and a TLS connector to use. Please refer to `connect_async()` for more details.
/// `disable_nagle` specifies if the Nagle's algorithm must be disabled, i.e.
/// `set_nodelay(true)`. If you don't know what the Nagle's algorithm is, better
/// leave it to `false`.
#[cfg(any(feature = "native-tls", feature = "__rustls-tls"))]
pub async fn connect_async_tls_with_config<R>(
    request: R,
    config: Option<WebSocketConfig>,
    disable_nagle: bool,
    connector: Option<Connector>,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error>
where
    R: IntoClientRequest + Unpin,
{
    let request = request.into_client_request()?;

    let domain = domain(&request)?;
    let port = request
        .uri()
        .port_u16()
        .or_else(|| match request.uri().scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(Error::Url(UrlError::UnsupportedUrlScheme))?;

    let addr = format!("{}:{}", domain, port);
    let try_socket = TcpStream::connect(addr).await;
    let socket = try_socket.map_err(Error::Io)?;

    if disable_nagle {
        socket.set_nodelay(true)?;
    }

    crate::tls::client_async_tls_with_config(request, socket, config, connector).await
}
