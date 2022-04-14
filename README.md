# cautious-axela

## Solenoid Info
- Uses a TI ULN2803A Darlington Transistor Array to Power Solenoid.
- Sets GPIO 50 to output.
- Grounded to DGRND.

## Browser Audio
Ensure web server is running
`$ cvlc connectBrowserAudio.sdp`

## Building
Ensure that the GLIBC version on the host and target platforms match up. Either use a docker with the proper GLIBC version to compile the code or update the image on the beaglebone. 

