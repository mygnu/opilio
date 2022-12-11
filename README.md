# Opilio

Opilio (a water crab) is a hobby project that brings high quality PC water cooling hub fully supported on linux.

Counterpart firmware for this project lives [here](https://github.com/mygnu/opilio-firmware) and open source pcb design files [here](https://github.com/mygnu/opilio-pcb)


## Key Features
- Works within PC tower or externally with 12v power adapter
- Configurable via USB-C interface
- Smart Mode for nice default settings
- 1 pump header and 5 Fan headers
- Operates independently of motherboard fan configuration (can be setup in another room, as long as a USB cable can reach it)
- Curve for temperature to speed for fans and pump (total 4, pump + 3 fans last 2 fans are combined)
- Auto shut off when PC is Off or goes to sleep (even when connect to external power)
- TUI interface
- JSON file based config

config is stored in `~/.config/opilio/opilio.json`
```json
{
  "general": {
    "sleep_after": 60
  },
  "smart_mode": {
    "trigger_above_ambient": 5.0,
    "upper_temp": 35.0,
    "pump_duty": 80.0
  },
  "settings": [
    {
      "id": "P1",
      "curve": [
        [10, 90],
        [20, 90],
        [30, 100],
        [40, 100]
      ]
    },
    {
      "id": "F1",
      "curve": [
        [20, 20],
        [25, 30],
        [30, 50],
        [40, 100]
      ]
    },
    {
      "id": "F2",
      "curve": [
        [20, 20],
        [25, 30],
        [30, 50],
        [40, 100]
      ]
    },
    {
      "id": "F3",
      "curve": [
        [20, 20],
        [25, 30],
        [30, 50],
        [40, 100]
      ]
    }
  ]
}
```

### General Setting

Currently this group only has one setting number of seconds to wait before system goes to sleep. This is useful when using external power, since you'd want to turn it off when there is no PC activity. You can choose to increase this time when going into the bios.

### Smart Mode: 
`smart_mode` is optional but defined by default. `trigger_above_ambient` is temperature in °C this decides the fan ON trigger point based on water IN sensor reading. i.e. if ambient is 22°C and fans will be turned ON when the water IN temperature reaches ambient + `trigger_above_ambient` or `22 + 5 = 27°C`. Fan speed is adjusted automatically and it will turn off if not required. `upper_temp` is the max temp that water is allowed to reach, beyond that all fans run at full throttle. Pump duty is a constant percentage speed of the pump. Pump is never turned off while system is polling Opilio via USB. 

### Settings P1, F[1-3]

these are temperature/speed curve definitions for the pump and fans. first parameter is temperature and second is speed in percentage. Only use if you really need to run pump/fans at different speed. Smart mode is quite powerful otherwise.

### TODO:
- GUI Interface
- Windows support (maybe)

### TUI Interface
![image](render/opilio-tui.gif)
