0.1.4
---
- MVP reached for API: implemented all basic methods for existing providers
- Feature flagged the alpaca provider. It is only necessary when callers use this API to interact with Alpaca. The main use case for the API is the simulated provider
- Cleaned up and restructured the API. For example order request and response structs were halved given that buy and sell orders have the same request and response fields as of now
- Provider and client implementations are now in util package
- API exposes everything an external caller may need to implement their own provider
- IronTradeClientProvider now has a default implementation for create client, which only requires the caller to pass in a builder
- Simulated broker now clearly makes a distinction between notional and non-notional assets. It now also supports currency (which is enforced to be one of the notional assets)
- Other changes for readability / functionality were done

0.1.3
----
- Added DO NOT USE to README

0.1.2
----
- Added simulated iron trade client
- Added documentation for trait IronTradeClient
- Added README content

0.1.1
-----
- Added AlpacaIronTradeClient with basic support for market buy/sell, orders and positions

0.1.0
-----
- Initial release