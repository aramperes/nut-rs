// use std::io::Read;
// use std::io::Write;
// use std::net::TcpStream;
//
// /// A Read implementation with optional SSL support.
// pub struct SslOptionalReader<'a> {
//     pub(crate) inner_stream: TcpStream,
//     #[cfg(feature = "ssl")]
//     ssl_stream: Option<rustls::Stream<'a, rustls::ClientSession, TcpStream>>,
// }
//
// impl<'a> SslOptionalReader<'a> {
//     #[cfg(feature = "ssl")]
//     pub fn new(inner: TcpStream) -> Self {
//         Self {
//             inner_stream: inner,
//             ssl_stream: None,
//         }
//     }
//
//     #[cfg(not(feature = "ssl"))]
//     pub fn new(inner: TcpStream) -> Self {
//         Self {
//             inner_stream: inner,
//         }
//     }
//
//     #[cfg(feature = "ssl")]
//     pub fn set_ssl_stream(
//         &mut self,
//         ssl_stream: rustls::Stream<'a, rustls::ClientSession, TcpStream>,
//     ) {
//         self.ssl_stream = Some(ssl_stream)
//     }
// }
//
// impl<'a> Read for SslOptionalReader<'a> {
//     #[cfg(feature = "ssl")]
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         if let Some(ssl_stream) = &mut self.ssl_stream {
//             ssl_stream.read(buf)
//         } else {
//             self.inner_stream.read(buf)
//         }
//     }
//
//     #[cfg(not(feature = "ssl"))]
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         self.inner_stream.read(buf)
//     }
// }
//
// impl<'a> Write for SslOptionalReader<'a> {
//     #[cfg(feature = "ssl")]
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         if let Some(ssl_stream) = &mut self.ssl_stream {
//             ssl_stream.write(buf)
//         } else {
//             self.inner_stream.write(buf)
//         }
//     }
//
//     #[cfg(not(feature = "ssl"))]
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         self.inner_stream.write(buf)
//     }
//
//     #[cfg(feature = "ssl")]
//     fn flush(&mut self) -> std::io::Result<()> {
//         if let Some(ssl_stream) = &mut self.ssl_stream {
//             ssl_stream.flush()
//         } else {
//             self.inner_stream.flush()
//         }
//     }
//
//     #[cfg(not(feature = "ssl"))]
//     fn flush(&mut self) -> std::io::Result<()> {
//         self.inner_stream.flush()
//     }
// }
//
// impl <'a> std::fmt::Debug for SslOptionalReader <'a> {
//     #[cfg(feature = "ssl")]
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "SslOptionalReader[ssl={}]", self.ssl_stream.is_some())
//     }
//
//     #[cfg(not(feature = "ssl"))]
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "SslOptionalReader[ssl={}]", false)
//     }
// }
