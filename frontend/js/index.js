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

let pc = new RTCPeerConnection({
  iceServers: [
    {
      urls: "stun:stun.l.google.com:19302"
    }
  ]
})

pc.ontrack = function (event) {
  const el = document.createElement(event.track.kind)
  el.srcObject = event.streams[0]
  el.autoplay = true
  el.controls = true

  cameras.appendChild(el)
}

pc.oniceconnectionstatechange = e => { console.log(e, pc.iceConnectionState) }
pc.onicecandidate = event => {
  if (event.candidate === null) {
    send("RtcSession", JSON.stringify(pc.localDescription), e => {
      console.log(e)
      //try {
      //  pc.setRemoteDescription(new RTCSessionDescription(JSON.parse(sd)))
      //} catch (e) {
      //  alert(e)
      //}
    })
  }
}

// Offer to receive 1 audio, and 2 video tracks
pc.addTransceiver("audio", { "direction": "recvonly" })
pc.addTransceiver("video", { "direction": "recvonly" })
pc.addTransceiver("video", { "direction": "recvonly" })

console.log("ws", "connecting")
socket.addEventListener("open", ev => {

  pc.createOffer()
    .then(d => pc.setLocalDescription(d))
    .catch((err) => {
      console.error(err);
    });
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

btn_lock.addEventListener("click", () => {
  send("Lock")
})


btn_ping.addEventListener("click", () => {
  send("Ping", "", resp => {
    console.log(resp)
  })
})
