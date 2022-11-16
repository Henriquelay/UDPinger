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

messageSize = 40

while True:
  message, address = serverSocket.recvfrom(messageSize)
  message = message.upper()
  print('Receive: ' + str(message))

  if sleep_for_rand_response_times:
         min_sleep = 0.2
         max_sleep = 2.0
         time.sleep(random.uniform(min_sleep, max_sleep))
         if simulate_packet_loss:
            if random.randint(0, 10) < 2:
               print ('Oops, I dropped, lol')
               continue

  serverSocket.sendto(message, address)
 