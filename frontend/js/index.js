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

const updatePinStatus = (code) => {
   code = JSON.parse(code);
   pin_number.innerHTML = "Current Pin: " + code.data;
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

scan_card.addEventListener("click", () => {
  document.getElementById("scan_card").innerText = "Scanning New Card...";
  send("NFCSet", "scanCard", _resp => {
    document.getElementById("scan_card").innerText = "Scan New Card";
    alert('Your card was added successfully.')
  })
})

show_card.addEventListener("click", () => {
  let msg = show_card.innerHTML;
  if (msg == 'Show'){
    send("NFCGet", msg, resp => {
      console.log(resp.response);
      let response = resp.response.slice(8, -2);
      document.getElementById("display_ids").innerText = response;
      show_card.innerHTML = "Hide";
      document.getElementById("display_ids").style.display = 'block';
    })
  }
  else {
    show_card.innerHTML = "Show";
    document.getElementById("display_ids").style.display = 'none';
  }
})

btn_ping.addEventListener("click", () => {
  send("Ping", "", resp => {
    console.log(resp)
  })
})

submit_new_pin.addEventListener("click", () => {
   let code = JSON.stringify({
      data: document.getElementById('pin_input').value
   });
   console.log(code);
   send("KeypadSetCode", code, _ => {
   })
})

// Timers
const getDoorStatus = () => {
  send("DoorGet", "", (resp) => {
    updateDoorStatus(resp.response);
  })
}
const doorStatusTimeout = setInterval(getDoorStatus, 1000);

const getKeyPadCode = () => {
  send("KeypadGetCode", "", (resp) => {
    updatePinStatus(resp.response);
  })
}
const keypadStatusTimeout = setInterval(getKeyPadCode, 1000);
