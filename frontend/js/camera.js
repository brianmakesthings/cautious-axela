let socket = new WebSocket(`ws://${location.host}/socket`)

let g_message_id = 0
const messages = {}
let stream = null;

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
  console.log("ws", ev)

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
  console.log(ev.track.kind)
  video.srcObject = ev.streams[0]
  video.autoplay = true
  video.controls = true
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

// Offer to receive 1 audio, and 2 video tracks
pc.addTransceiver("audio", { "direction": "recvonly" })
pc.addTransceiver("video", { "direction": "recvonly" })

function start_camera() {
  pc.createOffer()
    .then(d => pc.setLocalDescription(d))
    .catch((err) => {
      console.error(err);
    });
}

const AudioContext = window.AudioContext || window.webkitAudioContext;
const context = new AudioContext();
let apc = new RTCPeerConnection({
  iceServers: [
    {
      urls: "stun:stun.l.google.com:19302"
    }
  ]
})
apc.oniceconnectionstatechange = e => { console.log(e, pc.iceConnectionState) }
apc.onicecandidate = event => {
  if (event.candidate === null) {
    send("RtcAudioSession", JSON.stringify(pc.localDescription), e => {
      try {
        pc.setRemoteDescription(new RTCSessionDescription(JSON.parse(e.response)))
      } catch (e) {
        alert(e)
      }
    })
  }
}
mic_on.addEventListener("click", async () => {
  const constraints = { audio: true, video: false };
  try {
    // since we don't have HTTPS, go to about:config set to true media.devices.insecure.enabled and media.getusermedia.insecure.enabled
    stream = await navigator.mediaDevices.getUserMedia(constraints);
    const microphone = context.createMediaStreamSource(stream);
    apc.addStream(stream);
    let descriptor = await apc.createOffer();
    apc.setLocalDescription(descriptor);    
    // const filter = context.createBiquadFilter();
    // // microphone -> filter -> destination
    // microphone.connect(filter);
    // filter.connect(context.destination);
    // pc.addStream(stream)
    localAudio.srcObject = stream;
  } catch (err) {
    console.error("Failed to obtain user permission", err);
  }
})

mic_off.addEventListener("click", () => {
  stream.getTracks().forEach((track) => {
    track.stop();
  })
})