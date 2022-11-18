## Integration tests

This folder contains the integration tests for the Mage Hands crowdfunding platform.

### Preparation

To prepare tests download `snip20-reference-impl` (https://github.com/scrtlabs/snip20-reference-impl) in the mage-hands root directory.

Build the snip20-reference-impl, mage-hands-platform, and mage-hands-project contracts (see Makefile).

### Running tests

Change to `tests/` folder and run `npm install`.

Start the LocalSecret dev chain: 
`docker run -it --rm -p 9091:9091 -p 26657:26657 -p 1317:1317 -p 5000:5000 --name localsecret ghcr.io/scrtlabs/localsecret:v1.4.1-beta.3`

Run `npm start`

Note: SecretJS does not work with node 17, you must use node 16 instead.

### Seeding your wallet with SCRT

docker exec -it localsecret secretcli tx bank send a <YOUR_WALLET ADDRESS> 100000000uscrt