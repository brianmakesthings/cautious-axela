"use strict";
let socket = new WebSocket(`ws://${location.host}/socket`)

let g_message_id = 0
const messages = {}

function send(command, message = "", cb = () => { }) {
  if (socket.readyState != WebSocket.OPEN) {
    return
  }
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
  const msg = JSON.parse(ev.data)
  if (msg.id && msg.id in messages) {
    messages[msg.id](msg)
    delete messages[msg.id]
  }
}

socket.onopen = () => {
  connection_indicator.textContent = "Connected"
  btn_connect.disabled = false
}

// Helper Functions
const updateDoorStatus = (status) => {
  switch (status) {
    case "\"Lock\"":
      door_status.textContent = "Locked"
      break
    case "\"Unlock\"":
      door_status.textContent = "Unlocked"
      break
  }
}

const updatePinStatus = (code) => {
  code = JSON.parse(code)
  pin_number.textContent = code.data
}

// Event Listeners
btn_lock.addEventListener("click", () => {
  send("DoorSet", "\"Lock\"", resp => {
    updateDoorStatus(resp.response)
  })
})

btn_unlock.addEventListener("click", () => {
  send("DoorSet", "\"Unlock\"", resp => {
    updateDoorStatus(resp.response)
  })
})

scan_card.addEventListener("click", () => {
  document.getElementById("scan_card").innerText = "Scanning New Card..."
  send("NFCSet", "scanCard", _resp => {
    document.getElementById("scan_card").innerText = "Scan New Card"
    alert("Your card was added successfully.")
  })
})

show_card.addEventListener("click", () => {
  let msg = show_card.innerHTML
  if (msg == "Show") {
    send("NFCGet", msg, resp => {
      console.log(resp.response)
      let response = resp.response.slice(8, -2)
      document.getElementById("display_ids").innerText = response
      show_card.innerHTML = "Hide"
      document.getElementById("display_ids").style.display = "block"
    })
  }
  else {
    show_card.innerHTML = "Show"
    document.getElementById("display_ids").style.display = "none"
  }
})

btn_ping.addEventListener("click", () => {
  send("Ping", "", resp => {
    alert(JSON.stringify(resp))
  })
})

submit_new_pin.addEventListener("click", () => {
  let code = JSON.stringify({
    data: document.getElementById("pin_input").value
  });
  console.log(code)
  send("KeypadSetCode", code, _ => {
  })
})

// Timers
const getDoorStatus = () => {
  send("DoorGet", "", (resp) => {
    updateDoorStatus(resp.response)
  })
}

const doorStatusTimeout = setInterval(getDoorStatus, 1000)

const getKeyPadCode = () => {
  send("KeypadGetCode", "", (resp) => {
    updatePinStatus(resp.response)
  })
}

const keypadStatusTimeout = setInterval(getKeyPadCode, 1000)

/* camera related stuff */

function start_camera() {
  btn_connect.disabled = true

  let pc = new RTCPeerConnection({
    iceServers: [
      {
        urls: "stun:stun.l.google.com:19302"
      }
    ]
  })

  pc.ontrack = ev => {
    if (ev.track.kind == "video") {
      video.srcObject = ev.streams[0]
    }
  }

  pc.oniceconnectionstatechange = e => { console.log(e, pc.iceConnectionState) }
  pc.onicecandidate = event => {
    if (event.candidate === null) {
      send("RtcSession", JSON.stringify(pc.localDescription), e => {
        try {
          pc.setRemoteDescription(new RTCSessionDescription(JSON.parse(e.response)))
        } catch (e) {
          alert(e)
        }
      })
    }
  }

  pc.addTransceiver("video", { "direction": "recvonly" })
  pc.addTransceiver("audio", { "direction": "recvonly" })

  pc.createOffer()
    .then(d => pc.setLocalDescription(d))
    .catch((err) => {
      console.error(err)
    })
}


mic_on.addEventListener("click", async () => {
  let apc = new RTCPeerConnection({
    iceServers: [
      {
        urls: "stun:stun.l.google.com:19302"
      }
    ]
  })

  apc.oniceconnectionstatechange = e => { console.log(e, apc.iceConnectionState) }
  apc.onicecandidate = event => {
    if (event.candidate === null) {
      send("RtcAudioSession", JSON.stringify(apc.localDescription), e => {
        try {
          apc.setRemoteDescription(new RTCSessionDescription(JSON.parse(e.response)))
        } catch (e) {
          alert(e)
        }
      })
    }
  }
  // since we don't have HTTPS, go to about:config set to true media.devices.insecure.enabled and media.getusermedia.insecure.enabled
  navigator.mediaDevices
    .getUserMedia({ audio: true, video: false })
    .then(stream => {
      stream.getTracks().forEach(track => apc.addTrack(track, stream))
      localAudio.srcObject = stream
      apc.createOffer().then(descriptor => {
        apc.setLocalDescription(descriptor)
      }).catch(err => console.log(err))
    }).catch(err => console.log(err))
})

mic_off.addEventListener("click", () => {
  stream.getTracks().forEach((track) => {
    track.stop()
  })
})

btn_connect.addEventListener("click", () => {
  start_camera()
})
