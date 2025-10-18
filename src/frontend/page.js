let socket;
let passkey;
function send(data) {
    socket.send(`0 Control ${data}`);
}
function sendExt(data) {
    socket.send(`${passkey} ControlExt ${data}`);
}

const reconnect = (address) => {
    socket = new WebSocket(address);
    socket.addEventListener('close', () => {
        setTimeout(() => reconnect(address), 2500);
    });
}

window.lumineer = {
    main: (wsAddress, pass) => {
        passkey = pass;
        reconnect(wsAddress)
    },
    disableOutput: () => {
        sendExt('DisableOutput');
    },
    enableOutput: () => {
        sendExt('EnableOutput');
    },
    zero: () => {
        send('Zero');
    },
};