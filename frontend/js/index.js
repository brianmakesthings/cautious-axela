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

btn_lock.addEventListener("click", () => {
  send("lock")
})


btn_ping.addEventListener("click", () => {
  send("ping", "", resp => {
    console.log(resp)
  })
})
