require "keen"

Keen.project_id = "5ebc36e1a9c45677f158a665"
Keen.write_key = "f36899c9a85993e782196cf47ef6df7a88a9ef1b36f36e995530f4ef91c93b227374111cc145c06c6337da93fdd13d6b119d3a70b212622f21ee4c4950d44cacdaac1eeeb8643ec58b661f391c97be01f20b72c5141dce5f16708d7d3bd68fd4"

#/System/Library/PrivateFrameworks/Apple80211.framework/Resources/airport -I
Keen.publish(:pings, {
    AP => {
        :BSSID => "26:57:60:b5:d2:93",
        :SSID => "MOVISTAR_D28A",
        :Ping10Averagems => 10.1,
        :Ping10PercentLoss => 0.0
      },
    Probe => {
        :DeviceName => "Andrew's Mac Book Pro",
        :DeviceType => "laptop"
    }
  }
)
