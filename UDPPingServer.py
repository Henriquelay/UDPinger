# This has been provided by the class instructor and may not use the unlicense license.
from socket import *
import random, time

# What's your IP address and witch port should we use?
recieve_host = '127.0.0.1'
recieve_port = 30000

sleep_for_rand_response_times: bool = True
simulate_packet_loss: bool = True

# Create a UDP socket
# Notice the use of SOCK_DGRAM for UDP packets
serverSocket = socket(AF_INET, SOCK_DGRAM)
# Assign IP address and port number to socket
serverSocket.bind((recieve_host, recieve_port))

headerSize = 10
payloadSize = 30
messageSize = headerSize + payloadSize

while True:
   message, address = serverSocket.recvfrom(messageSize)
   message = message.upper()
   # print('Receive: ')
   print(str(message))
   seq = int.from_bytes(message[0:5], byteorder='little')
   print('Seq: ' + str(seq))
   message = bytearray(message)

   if simulate_packet_loss:
      if random.randint(0, 10) < 2:
         print ('Oops, I dropped, lol')
         continue

   print('Returning after ', end='')
   if sleep_for_rand_response_times:
      min_sleep = 0.2
      max_sleep = 1.0
      sleep_for = random.uniform(min_sleep, max_sleep)
      time.sleep(sleep_for)
      print(f'{sleep_for:.2f}', end='')
   else:
      print('0.00' , end='')
   print(' secs')

   if random.randint(0, 10) < 3:
      # Set wrong Pong type
      message[5:6] = [2]
      print ('Setting wrong Pong msg')
   else:
      # Set correct Pong type
      message[5:6] = [0]
 
   # if random.randint(0, 10) < 3:
   #    # Destroy payload
   #    message[headerSize:payloadSize] = random.randint(0, 255).to_bytes(payloadSize, byteorder='little')
   #    print ('Setting corrupted payload')

   message = bytes(message)
   serverSocket.sendto(message, address)
 