import numpy as np
import math
from . import user_namespace as un

import logging
logger = logging.getLogger('prefigure')

# introduce some useful mathematical operations
#   that are meant to be available to authors

def dot(u, v):
    return np.dot(np.array(u), np.array(v))

def length(u):
    return np.linalg.norm(np.array(u))

def normalize(u):
    return 1/length(u) * np.array(u)

def midpoint(u, v):
    return 0.5*(np.array(u) + np.array(v))

def angle(p, units = 'deg'):
    angle = math.atan2(p[1], p[0])
    if units == 'deg':
        return math.degrees(angle)
    return angle

def roll(array):
    array = np.array(array)
    return np.roll(array, 1, axis=0)

def choose(n,k):
    n = int(n)
    k = int(k)
    return math.comb(n, k)

def append(input, item):
    l = list(input)
    l.append(item)
    if isinstance(input, np.ndarray):
        l = np.array(l)
    return l

def chi_oo(a, b, t):
    if t > a and t < b:
        return 1
    return 0

def chi_oc(a, b, t):
    if t > a and t <= b:
        return 1
    return 0

def chi_co(a, b, t):
    if t >= a and t < b:
        return 1
    return 0

def chi_cc(a, b, t):
    if t >= a and t <= b:
        return 1
    return 0

def rotate(v, theta):
    c = math.cos(theta)
    s = math.sin(theta)
    return np.array([c*v[0]-s*v[1], s*v[0]+c*v[1]])

def zip_lists(a, b):
    return list(zip(a, b))

def filter(df, a, b, value):
    mask = df[b] == value
    return df[a][mask]

def evaluate_bezier(controls, t):
    dim = len(controls[0])
    N = len(controls)
    controls = np.array(controls)
    sum = np.array([0.0] * dim)
    if N == 3:
        coefficients = [1,2,1]
    else:
        coefficients = [1,3,3,1]
    for j in range(N):
        sum += coefficients[j] * (1-t)**(N-j-1) * t**j * controls[j]
    return sum

def eulers_method(f, t0, y0, t1, N):
    h = (t1 - t0)/N
    if isinstance(y0, np.ndarray):
        points = [[t0, *y0]]
    else:
        points = [[t0, y0]]
    t = t0
    y = y0
    for _ in range(N):
        y += f(t, y) * h
        t += h
        if isinstance(y, np.ndarray):
            points.append([t, *y])
        else:
            points.append([t, y])
    return np.array(points)

# dirac delta function to be used in solving ODEs
def delta(t, a):
    breaks = un.retrieve('__breaks')
    if breaks is not None:
        breaks.append(a)
        return 0
    delta_on = un.retrieve('__delta_on')
    if np.isclose(t, a) and delta_on:
        return 1

    return 0

