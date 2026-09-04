Output assets/demo.gif

Set Shell "bash"
Set FontSize 18
Set Width 1600
Set Height 1000
Set Padding 20
Set Theme "ayu"

Require tuiql

Set TypingSpeed 90ms

Type@100ms "tuiql open pagila"
Enter
Sleep 1s

Type@90ms ":"
Sleep 0.5s
Type@90ms "goto "
Tab@230ms 7
Enter
Sleep 1s

Type@130ms ":"
Sleep 0.5s
Type@130ms "order-by country asc"
Enter
Sleep 1.4s

Type@120ms ":where country_id >= 100"
Enter
Sleep 1.5s

Type@130ms ":"
Sleep 0.5s
Type@120ms "save-preset by-country"
Enter
Sleep 1.4s

Down@200ms 3
Sleep 800ms

Enter
Sleep 1s

Type@150ms "y"
Sleep 1s

Type@100ms ":help"
Enter
Sleep 3.5s
