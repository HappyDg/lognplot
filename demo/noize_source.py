""" Test noise source, which transmits a sine wave
over TCP/IP
"""

import math
import time

from lognplot.client import LognplotTcpClient


def main():
    t = 0.0
    A = 10.0  # Sine wave amplitude [-]
    F = 1.3  # Sine wave frequency [Hz]
    A2 = 1.2
    F2 = 100
    B = 5.0  # Sine wave offset
    client = LognplotTcpClient()
    client.connect()

    dt = 0.0001  # 10 kHz
    n_samples = 2000
    while True:
        samples = []
        samples2 = []
        t0 = t
        # Generate samples:
        for _ in range(n_samples):
            omega = 2 * math.pi * F
            omega2 = 2 * math.pi * F2
            sample = A * math.sin(omega * t) + B + A2 * math.cos(omega2 * t)
            sample2 = A * math.sin(omega * t) + B + A2 * math.cos(omega2 * t) + 9
            samples.append(sample)
            samples2.append(sample2)

            # Increment time:
            t += dt

        print(f"Sending {len(samples)} samples")
        client.send_samples("Trace1", t0, dt, samples)
        client.send_samples("Trace2", t0, dt, samples2)

        time.sleep(n_samples * dt)


if __name__ == "__main__":
    main()
