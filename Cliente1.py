#!/usr/bin/env python
import socket, signal, sys, time, numpy as np

# Initial configurations
msgType = 0                    # 0 = ping, 1 = pong
msgFromClient = "Bruno Missi Xavier" # Original message string
numberPings = 10               # Number of times that will Ping

#serverAddressPort = ("152.67.56.170",54321) # Server IP Address and service port
serverAddressPort = ("168.227.188.22",30000)      # Server IP Address and service port
bufferSize = 40                              # Buffer size (bytes)

# Create an UDP socket in client side
UDPClientSocket = socket.socket(family=socket.AF_INET, type=socket.SOCK_DGRAM)
UDPClientSocket.settimeout(1.0) # Set max timeout

# Data
min_rtt = 0
max_rtt = 0
avg_rtt = 0
mdev_rtt = 0
total_packets = 0
dropped_packets = 0
error_packets = 0

dataSendArray = []
timeResponseArray = []
actualPackage = 0

def packageNumber(response):
    if(response[0:5].isnumeric()):
        return int(response[0:5])
    else:
        print("Error: Package number does not conform to specification: {}.".format(response[0:5]))
        raise Exception("Error: Package number does not conform to specification: {}.".format(response[0:5]))


def msgTypeReturn(response):
    if(response[5].isnumeric()):
        return response[5]
    else:
        print("Error: Package type does not conform to specification: {}.".format(response[5]))
        raise Exception("Error: Package type does not conform to specification: {}.".format(response[5]))
    

def stampTime(finalTime,response):
    if(response[6:10].isnumeric()):
        stampTime = (int(str(finalTime)[11:15]) - int(response[6:10]))
        if stampTime<0:
            stampTime= stampTime+10000
        return stampTime/10.0
    else:
        print("Error: Package timestamp does not conform to specification: {}.".format(response[6:10]))
        raise Exception("Error: Package timestamp does not conform to specification: {}.".format(response[6:10]))
    


def printResults():
    global min_rtt, max_rtt, avg_rtt, mdev_rtt, total_packets, dropped_packets, numberPings, timeResponseArray

    print("Results:")
    print("min_rtt:{:.2f} ms, max_rtt:{:.2f} ms, avg_rtt:{:.2f} ms, mdev_rtt:{:.2f} ms".format(min_rtt, max_rtt, avg_rtt, mdev_rtt))
    print("total_packets:{}, dropped_packets:{} ({:.2f}%), error_packets:{} ({:.2f}%)".format(total_packets, dropped_packets, (100*dropped_packets/total_packets), error_packets, (100*error_packets/total_packets)))


def calculateData():
    global min_rtt, max_rtt, avg_rtt, mdev_rtt, total_packets, dropped_packets, numberPings, timeResponseArray

    array = np.asarray(timeResponseArray)
    if(array.size > 0):
        min_rtt = array.min()
        max_rtt = array.max()
        avg_rtt = array.mean()
        mdev_rtt = array.std()
    total_packets = numberPings

def signal_handler(sig, frame):
    global min_rtt, max_rtt, avg_rtt, mdev_rtt, actualPackage, total_packets, dropped_packets, numberPings, timeResponseArray
    actualPackage = actualPackage - 1
    calculateData()

    print("\nResults:")
    print("min_rtt:{:.2f} ms, max_rtt:{:.2f} ms, avg_rtt:{:.2f} ms, mdev_rtt:{:.2f} ms".format(min_rtt, max_rtt, avg_rtt, mdev_rtt))
    if(actualPackage!=0):
        print("total_packets:{}, dropped_packets:{} ({:.2f}%), error_packets:{} ({:.2f}%)".format(actualPackage, dropped_packets, (100*dropped_packets/actualPackage), error_packets, (100*error_packets/actualPackage)))
    else:
        print("No one packet has been send...")
    sys.exit(0)


def wait_response(bytesSent):
    global dropped_packets, error_packets, bufferSize, UDPClientSocket, timeResponseArray, numberPings, actualPackage
    while True:
        try:
            msgFromServer = UDPClientSocket.recvfrom(bufferSize)
            msgFromServer = msgFromServer[0].decode('utf-8')
            finalTime =  time.time()

            received_packN = packageNumber(msgFromServer)
            received_msgType = msgTypeReturn(msgFromServer)
            received_stime = stampTime(finalTime,msgFromServer)

            if (received_packN < 0 or received_packN > numberPings) :
                print("Error: Package number out of range.", received_packN)
                error_packets = error_packets + 1
                return "Error: Package number out of range."
            elif(actualPackage != received_packN):
                print("Error: Package number ({}) is not the same as the one sent ({}).".format(received_packN, actualPackage), received_packN)
                error_packets = error_packets + 1
                return "Error: Package number ({}) is not the same as the one sent ({}).".format(received_packN, actualPackage)
            elif (received_msgType!='1'):
                print("Error: Type different from expected (Pong = 1).", str(msgFromServer[5]))
                error_packets = error_packets + 1
                return "Error: Type different from expected (Pong = 1)."
            elif (bytesSent.decode()[6:10] != msgFromServer[6:10]):
                print("Error: Corrupted timestamp, expected {} but received {}.".format(bytesSent.decode()[6:10],msgFromServer[6:10]))
                error_packets = error_packets + 1
                return "Error: Corrupted timestamp, expected {} but received {}.".format(bytesSent.decode()[6:10],msgFromServer[6:10])
 
            print("package={} time={:.2f} ms".format(received_packN,received_stime))
            timeResponseArray.append(received_stime)
            return msgFromServer

        except socket.timeout:
            dropped_packets=dropped_packets+1
            print("Error: Timed out.")
            return "Error: Timed out."

        except Exception as err:
            error_packets = error_packets + 1
            return "Error: {}.".format(err)

            
def send_message(bytesToSend,wait=False):
    global serverAddressPort, UDPClientSocket, actualPackage
    UDPClientSocket.sendto(bytesToSend,serverAddressPort)
    actualPackage=actualPackage+1

    if wait == False:
        return
    else:
        return wait_response(bytesToSend)

def main():
    signal.signal(signal.SIGINT, signal_handler)

    print("PING to {}".format(serverAddressPort))
    for counter in range(numberPings):
        counter=counter+1
        initialTime = time.time()
        bytesToSend = str.encode(str(counter).zfill(5)+str(msgType)+str(initialTime)[11:15]+msgFromClient) # mensagem encodada para bytes
        response = send_message(bytesToSend,True)


    calculateData()
    printResults()

if __name__ == '__main__':
    main()
