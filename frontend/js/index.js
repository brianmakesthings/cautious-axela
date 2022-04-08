"use strict";
let socket = new WebSocket(`ws://${location.host}/socket`)

let g_message_id = 0
const messages = {}

function send(command, message = "", cb = () => { }) {
  let message_id = `${g_message_id++}`
  socket.send(JSON.stringify({
    "id": message_id,
    "command": command,
    "message": message
  }))

  messages[message_id] = cb
}

console.log("ws", "connecting")
socket.addEventListener("open", ev => {
  console.log("ws", "connected", ev)
})

socket.onmessage = ev => {
  console.log("ws", ev)

  const msg = JSON.parse(ev.data)
  if (msg.id && msg.id in messages) {
    messages[msg.id](msg)
    delete messages[msg.id]
  }
}

// Helper Functions
const updateDoorStatus = (status) => {
  let statusText = "";
  switch (status) {
    case "\"Lock\"":
      statusText = "Locked";
      break;
    case "\"Unlock\"":
      statusText = "Unlocked";
      break;
  }
  door_status.innerHTML = "Status: " + statusText;
}

// Event Listeners
btn_lock.addEventListener("click", () => {
  send("DoorSet", "\"Lock\"", resp => {
    updateDoorStatus(resp.response);
  })
})

btn_unlock.addEventListener("click", () => {
  send("DoorSet", "\"Unlock\"", resp => {
    updateDoorStatus(resp.response);
  })
})

btn_ping.addEventListener("click", () => {
  send("Ping", "", resp => {
    console.log(resp)
  })
})

// Timers
const getDoorStatus = () => {
  send("DoorGet", "", (resp) => {
    updateDoorStatus(resp.response);
  })
}
const doorStatusTimeout = setInterval(getDoorStatus, 1000);