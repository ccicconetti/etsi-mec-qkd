# MEO

Below we describe the operations of a basic MEO.

Assumptions:

- there are N > 1 platforms that can serve the incoming client apps 
- for each platform the MEO has the following instantaneous telemetry data:
  - [0,1] load of each platform
  - [0,1] availability of secret material through QKD on the key manager of the platform

Parameters:

- maximum load and minimum key availability to consider a platform as a candidate
- score assigning function

## New application context

When the MEO receives a request to add a new application context it performs the following operations:

1. Filter the platforms by checking the blacklist/whitelist
2. Remove from the set of candidate platforms those with load too high or key availability too low
3. If the candidate set is empty, the application context is rejected (and the LCMP is notified accordingly)
4. Otherwise, the application context can be admitted and it is assigned a unique `contextId`
5. Assign a score to each platform in the candidate set based on the load and key availability
6. Select the target platform at random using the score as a weight, i.e., the higher the score of a platform, the better the chances that the platform is selected
7. If there is no active context for the given application on the selected platform, perform a deployment operation
8. Add the new context to the KVS
9. Reply to the LCMP with the `contextId` newly assigned and the `referenceURI` of the target platform.

## Delete application context

When the MEO receives a command to terminate an application context it performs the following operations

1. Remove the application context from the KVS
2. If the platform does not have any other active context using the same application, remove it from the platform
## Periodic optimization

Periodically the MEO performs the following operations:

1. Check the load and key availability on all platforms
2. If there is a platform with a load higher than the maximum allowed or a key availability lower than the minimum allowed, perform the migration of one application to another platform
3. Select the target platform using the same procedure as in _new application context_ above
4. If there is no available target platform, then exit immediately (and possibly notify an overload condition to an external monitoring system)
5. Otherwise, update the KVS with the migration from origin to target platform
6. If there are no other active contexts in the origin platform using the same application, remove it from the platform
7. If there is no active context for the application on the target platform, perform a deployment operation
8. Inform the LCMP of the new `referenceURI` to be notified to the device app for the application context migrated

