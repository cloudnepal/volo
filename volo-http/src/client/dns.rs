//! DNS resolver implementation
//!
//! This module implements [`DnsResolver`] as a [`Discover`] for client.

use std::{net::SocketAddr, ops::Deref, sync::Arc};

use async_broadcast::Receiver;
use faststr::FastStr;
use hickory_resolver::{
    config::{LookupIpStrategy, ResolverConfig, ResolverOpts},
    AsyncResolver, TokioAsyncResolver,
};
use volo::{
    context::Endpoint,
    discovery::{Change, Discover, Instance},
    loadbalance::error::LoadBalanceError,
    net::Address,
};

use crate::error::client::{bad_host_name, no_address};

/// The port for `DnsResolver`, and only used for `DnsResolver`.
///
/// When resolving domain name, the response is only an IP address without port, but to access the
/// destination server, the port is needed.
///
/// For setting port to `DnsResolver`, you can insert it into `Endpoint` of `callee` in
/// `ClientContext`, the resolver will apply it.
#[derive(Clone, Copy, Debug, Default)]
pub struct Port(pub u16);

impl Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A service discover implementation for DNS.
#[derive(Clone)]
pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    /// Build a new `DnsResolver` through `ResolverConfig` and `ResolverOpts`.
    ///
    /// For using system config, you can create a new instance by `DnsResolver::default()`.
    pub fn new(config: ResolverConfig, options: ResolverOpts) -> Self {
        Self {
            resolver: AsyncResolver::tokio(config, options),
        }
    }

    /// Resolve a host to an IP address and then set the port to it for getting an [`Address`].
    pub async fn resolve(&self, host: &str, port: u16) -> Option<Address> {
        // Note that the Resolver will try to parse the host as an IP address first, so we don't
        // need to parse it manually.
        let mut iter = self.resolver.lookup_ip(host).await.ok()?.into_iter();
        Some(Address::Ip(SocketAddr::new(iter.next()?, port)))
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        let (conf, mut opts) = hickory_resolver::system_conf::read_system_conf()
            .expect("[Volo-HTTP] DnsResolver: failed to parse dns config");
        if conf
            .name_servers()
            .first()
            .expect("[Volo-HTTP] DnsResolver: no nameserver found")
            .socket_addr
            .is_ipv6()
        {
            // The default `LookupIpStrategy` is always `Ipv4thenIpv6`, it may not work in an IPv6
            // only environment.
            //
            // Here we trust the system configuration and check its first name server.
            //
            // If the first nameserver is an IPv4 address, we keep the default configuration.
            //
            // If the first nameserver is an IPv6 address, we need to update the policy to prefer
            // IPv6 addresses.
            opts.ip_strategy = LookupIpStrategy::Ipv6thenIpv4;
        }
        Self::new(conf, opts)
    }
}

impl Discover for DnsResolver {
    type Key = FastStr;
    type Error = LoadBalanceError;

    async fn discover<'s>(
        &'s self,
        endpoint: &'s Endpoint,
    ) -> Result<Vec<Arc<Instance>>, Self::Error> {
        if endpoint.service_name_ref().is_empty() && endpoint.address().is_none() {
            tracing::error!("[Volo-HTTP] DnsResolver: no domain name found");
            return Err(LoadBalanceError::Discover(Box::new(no_address())));
        }
        if let Some(address) = endpoint.address() {
            let instance = Instance {
                address,
                weight: 10,
                tags: Default::default(),
            };
            return Ok(vec![Arc::new(instance)]);
        }
        let port = match endpoint.get::<Port>() {
            Some(port) => port.0,
            None => {
                unreachable!();
            }
        };

        if let Some(address) = self.resolve(endpoint.service_name_ref(), port).await {
            let instance = Instance {
                address,
                weight: 10,
                tags: Default::default(),
            };
            return Ok(vec![Arc::new(instance)]);
        };
        tracing::error!("[Volo-HTTP] DnsResolver: no address resolved");
        Err(LoadBalanceError::Discover(Box::new(bad_host_name())))
    }

    fn key(&self, endpoint: &Endpoint) -> Self::Key {
        endpoint.service_name()
    }

    fn watch(&self, _: Option<&[Self::Key]>) -> Option<Receiver<Change<Self::Key>>> {
        None
    }
}
