Ein Programm welches Daten von einem verbundenen Arduino empfangen und als Graph darstellen kann, geschrieben in Rust.

# Installation
1. Rust installieren: 
Anweisungen auf https://rustup.rs/

2. Das Programm installieren:
Folgenden Befehl im Terminal ausführen:
```bash
cargo install --git https://github.com/Einfachirgendwa1/arduinographplotter
```

# Benutzung
Zum Starten einfach:
```bash
ArduinoGraphPlotter
```
Auf Windows *muss* und auf Linux & MacOs kann der Pfad zum Arduino angegeben werden:
Zum herausfinden einfach die Arduino IDE öffnen, und dann "Select Board" anklicken.
Unter dem Namen des Arduino sollte etwas stehen wie /dev/ttyACM0 auf Linux / MacOS oder COM*zahl* auf Windows.
Dann einfach:
```bash
ArduinoGraphPlotter /dev/...
```
bzw.
```bash
ArduinoGraphPlotter COM...
```

# Noch zu tun
1. Achsenbeschriftung (wird vermutlich schwierig, mal schauen)
2. Das Ganze schöner machen (wird auch schwierig, ich bin kein Grafikdesigner)
3. Bessere Fehlerbehandlung, kein .unwrap() überall
