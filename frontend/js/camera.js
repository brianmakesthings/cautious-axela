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

socket.onmessage = ev => {
  const msg = JSON.parse(ev.data)
  if (msg.id && msg.id in messages) {
    messages[msg.id](msg)
    delete messages[msg.id]
  }
}

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

function start_camera() {
  btn_connect.disabled = true
  pc.createOffer()
    .then(d => pc.setLocalDescription(d))
    .catch((err) => {
      console.error(err);
    });
}

btn_connect.addEventListener("click", () => {
  start_camera()
})

socket.onopen = () => {
  btn_connect.disabled = false
}
