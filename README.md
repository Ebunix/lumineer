# Lumineer
Lumineer is a server designed for giving granular control over a lighting rig to party visitors. It's primarily designed for the Demoscene, where requests have been made to incorporate the lighting of the party hall into productions.

# How to use
How you use Limuneer depends on who you actually are. Overall it is designed to be easy to use by party visitors without much of a hassle.

## As visitor
You either connect your demo via WebSocket (easiest for web-based demos) or send data directly to a UDP port on the Lumineer server. By default, the WebSocket port is 8000, and the UDP port is 8005.

## As operator
You set up an instance of Lumineer on your lighting network. This requires a bit of consideration, as Lumineer needs to be able to send ArtNET packets to your lighting setup (other protocols may follow, but ArtNET is the most widespread). Lumineer currently does not work as a middleman or merger between your lighting console and your rig, so you will need a merger or node that can handle two separate ArtNET inputs. The configuration file included with Lumineer allows you to customize the setup and ports to your liking.

# Visitor protocol
Lumineer uses a text-based protocol to deliver lighting data. It's built in a way where the visitors don't
need to know about the inner workings of the lighting rig, the DMX protocol or exact addresses of any lighting fixtures in the hall. Addresses and exposed parameters are set up before the party by the organizers, and each fixture is assigned a unique ID. This ID is then used to address each fixture, making it possible to change their physical address without breaking existing configurations on the visitor side.

A single update packet can consist of multiple fixture entries. A single fixture entry looks like this:
```
<ID> <Feature> <Value>\n
```
Where `ID` is a numeric identifier that uniquely identifies a single fixture. In the physical lighting rig, an ID maps to a DMX address somewhere in the setup, but the exact address is not relevant to the visitor. Also note the `\n` at the end of the line. Each line of the update packet contains data to update a single fixture, but one packet may contain an arbitrary amount of lines. Each line has to be terminated with a newline (`\n`).

The `Feature` of the update is a string identifier, telling Lumineer which parameter of the fixture to update. This also abstracts away the need to understand the difference between 8bit and 16bit parameters, as this is handled automatically for you by Lumineer.

The `Value` of the update is a numeric representation of the desired target value for the parameter. This may either be a floating point number or an unsigned integer within the range [0, 255]. In general, floating point representation is recommended for precision reasons, unless you want to set a parameter to an exact value. For features like Dimmer or RGB values the exact values usually matter a lot less than for Gobo channels, for example. Floating point numbers are required to include a period (.) to be recognized as such. Otherwise, they might get wrongly interpreted as integers.

A full update packet therefore might look like this:
```
101 Dim 1.0
101 ColorR 1.0
101 ColorG 0.5
101 ColorB 0.0
102 Dim 0.5
102 ColorR 0.0
102 ColorG 0.0
102 ColorB 1.0
```
This packet will set fixture 101 to 100% intensity with an orange color, and fixture 102 to 50% intensity with a blue color.

One thing to keep in mind with movable lighting fixtures is that by default they __don't point straight__, meaning when you don't set up their pan and tilt values, they will probably point either at the ceiling or at the floor, depending on how how they're placed. In general, if you want them to point straight away from the fixure, consider a Pan and Tilt value of 0.5 the center. You should follow the assumption that with a Pan of 0.5, Tilt values < 0.5 will point the fixture towards the crowd, and values > 0.5 will progressively point it towards the screen.

Some features can additinally be addressed using named values. Therefore a fixture with a color wheel might have a named value "Red", indicating that this will set the color wheel to a red color. The command for this would look like `101 ColorWheel Red`, or likewise for an indexed gobo wheel it could look like `101 Gobo1 Index6`. There are no quotation marks needed for the named value, and they are always written without spaces.

Your demo should make sure to signal the end of any lighting output by sending the special control packet `0 Control End`. This makes sure any remaining output gets properly cleared. Failing to do so might leave settings in an inconsistent state for the next user. You only need to do this once at the end of your demo. You may also use `0 Control Zero` at any time to set the entire output to 0 at once.

Note that any feature names that aren't known in Lumineer's configuration will be silently discarded.

# Examples
## JavaScript WebSockets
```js
// This address will be provided to you before the party
const LUMINEER_ADDRESS = "ws://2.0.0.4:8000"
const socket = new WebSocket(LUMINEER_ADDRESS);

socket.addEventListener('open', () => {
    socket.send(Buffer.from(`
        101 Dim 1.0
        101 ColorR 1.0
        101 ColorG 0.5
        101 ColorB 0.0
        102 Dim 0.5
        102 ColorR 0.0
        102 ColorG 0.0
        102 ColorB 1.0
    `));
});

```
## Node.js UDP
```js
const dgram = require('node:dgram');

// This address and port will be provided to you before the party
const LUMINEER_ADDRESS = "2.0.0.4";
const LUMINEER_PORT = 8005;

const udpSocket = dgram.createSocket('udp4');
const data = Buffer.from(`
    101 Dim 1.0
    101 ColorR 1.0
    101 ColorG 0.5
    101 ColorB 0.0
    102 Dim 0.5
    102 ColorR 0.0
    102 ColorG 0.0
    102 ColorB 1.0
`);
udpSocket.send(data, LUMINEER_PORT, LUMINEER_ADDRESS);

```
