# Connect to gdb remote server
target extended-remote :3333

# Load will flash the code
load

# Eanble demangling asm names on disassembly
set print asm-demangle on

# Enable pretty printing
set print pretty on

# Disable style sources as the default colors can be hard to read
set style sources off

# set backtrace limit to not have infinite backtrace loops
# set backtrace limit 32

# Initialize monitoring so iprintln! macro output
# is sent from the itm port to itm.txt
# monitor tpiu config internal itm.txt uart off 8000000
# Turn on the itm port
# monitor itm port 0 on

monitor arm semihosting enable

# Set a breakpoint at main, aka entry
break main

# Set a breakpoint at DefaultHandler
break DefaultHandler

# Set a breakpiont at HardFault
break HardFault

# Continue running and until we hit the main breakpoint
continue

# Step from the trampoline code in entry into main
step






# # send captured ITM to the file itm.fifo
# # (the microcontroller SWO pin must be connected to the programmer SWO pin)
# # 8000000 must match the core clock frequency
# monitor tpiu config internal itm.txt uart off 8000000

# # OR: make the microcontroller SWO pin output compatible with UART (8N1)
# # 8000000 must match the core clock frequency
# # 2000000 is the frequency of the SWO pin
# monitor tpiu config external uart off 8000000 2000000

# # enable ITM port 0
# monitor itm port 0 on


