Rust redis load limiter

This is a demonstration (only) of doing load limiting for say an endpoint, but using redis.

The general technique came from ... some website that I should have written down (update this)

The technique is as follows:

Given a period of time (in the example a minute), create a hash in redis that is broken down by some sub-division of that unit (in this case, seconds).
As a request comes in, incr the key corresponding to the current subdivision (second)
Get the whole mess
total up the accesses in the full thing, excepting any values that have aged out.  so basically add up the last 60 seconds worth.
Are you over the limit?  Refuse this one.
Other wise, allow it.

We then do a cleanup of old keys as well just to reduce the total amount of stuff we need to getall()

Right now it is in a proof of concept form of a dang ole function that does a thing rather than a type that ... does the thing.

 
