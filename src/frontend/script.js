// Connecting to the WebSocket server
const socket = new WebSocket("ws://127.0.0.1:8080/ws");

// HTML elements
const messageList = document.getElementById("chat-list");
const messageInput = document.getElementById("message-input");
const sendButton = document.getElementById("send-btn");

// User name
let userName;

// WebSocket event handlers
socket.addEventListener("open", () => {
    console.log("Connected to WebSocket server");

    // Join default room with random name
    userName = (Math.random() + 1).toString(36).substring(7);
    const nameMsg = "/name " + userName;
    socket.send(nameMsg);
    socket.send("/join chat");
});

socket.addEventListener("message", (event) => {
    console.log("Message received:", event.data);

    let message = event.data;
    
    // If message is in bytes, decode it
    if (event.data instanceof Blob) {
        const reader = new FileReader();
        reader.onloadend = function() {
            message = reader.result;
            appendReceivedMessage(message);
        };
        reader.readAsText(event.data);
    } else if (event.data instanceof ArrayBuffer) {
        const decoder = new TextDecoder();
        message = decoder.decode(event.data);
        appendReceivedMessage(message);
    } else {
        appendReceivedMessage(message); // If it's a regular text message
    }
});

socket.addEventListener("close", () => {
    console.log("Disconnected from WebSocket server");
});

socket.addEventListener("error", (error) => {
    console.error("WebSocket error:", error);
});

// Sending a message to the server
sendButton.addEventListener("click", () => {
    const message = messageInput.value.trim();
    if (message) {
        console.log("Message is sent");
        socket.send(message);
        appendSentMessage(message);
        messageInput.value = ""; // Clear input box
    }
});

// Append received message with username and message
function appendReceivedMessage(message) {
    const [username, ...msgParts] = message.split(":");
    const msgContent = msgParts.join(":").trim(); // In case there's extra spaces after the colon

    if (!message.startsWith("[")) {
        appendInfoMessage(message);
    } else {
        // Regular user message
        let messageElement = document.createElement("p");
        messageElement.textContent = `${username}: ${msgContent}`;

        let currentTime = new Date().toLocaleTimeString();
        let timeElement = document.createElement("span");
        timeElement.setAttribute("class", "timestamp");
        timeElement.textContent = currentTime;

        let messageContainer = document.createElement("div");
        messageContainer.setAttribute("class", "message-content");

        messageContainer.appendChild(messageElement);
        messageContainer.appendChild(timeElement);

        let messageReceived = document.createElement("div");
        messageReceived.setAttribute("class", "message received");
        messageReceived.appendChild(messageContainer);

        messageList.appendChild(messageReceived);
        messageList.scrollTop = messageList.scrollHeight;
    }
}

// Append sent message
function appendSentMessage(message) {
    let messageElement = document.createElement("p");
    messageElement.textContent = `You: ${message}`;

    let currentTime = new Date().toLocaleTimeString();
    let timeElement = document.createElement("span");
    timeElement.setAttribute("class", "timestamp");
    timeElement.textContent = currentTime;

    let messageContainer = document.createElement("div");
    messageContainer.setAttribute("class", "message-content");

    messageContainer.appendChild(messageElement);
    messageContainer.appendChild(timeElement);

    let messageSent = document.createElement("div");
    messageSent.setAttribute("class", "message sent");
    messageSent.appendChild(messageContainer);

    messageList.appendChild(messageSent);
    messageList.scrollTop = messageList.scrollHeight;
}

// Append info message (admin or system message)
function appendInfoMessage(message) {
    let messageElement = document.createElement("p");
    messageElement.textContent = message;

    let currentTime = new Date().toLocaleTimeString();
    let timeElement = document.createElement("span");
    timeElement.setAttribute("class", "timestamp");
    timeElement.textContent = currentTime;

    let messageContainer = document.createElement("div");
    messageContainer.setAttribute("class", "message-info");

    messageContainer.appendChild(messageElement);
    messageContainer.appendChild(timeElement);

    let infoMessage = document.createElement("div");
    infoMessage.setAttribute("class", "message info");
    infoMessage.appendChild(messageContainer);

    messageList.appendChild(infoMessage);
    messageList.scrollTop = messageList.scrollHeight;
}
