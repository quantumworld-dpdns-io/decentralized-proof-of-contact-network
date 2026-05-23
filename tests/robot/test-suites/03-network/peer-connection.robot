*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/network.resource
Suite Setup    Setup Test Environment
Suite Teardown    Stop All Running Nodes
Test Tags    network    peer    regression

*** Test Cases ***
Connect Two Peers
    [Tags]    smoke    network
    ${result}=    Connect Peers    node-peer-1    node-peer-2
    Should Be Equal    ${result}[status]    connected

Peers Are Connected Bidirectionally
    [Tags]    network
    Connect Peers    node-peer-bidi-1    node-peer-bidi-2
    Peer Should Be Connected    node-peer-bidi-1    node-peer-bidi-2
    Peer Should Be Connected    node-peer-bidi-2    node-peer-bidi-1

Disconnect Peers
    [Tags]    network
    Connect Peers    node-peer-disc-1    node-peer-disc-2
    Disconnect Peers    node-peer-disc-1    node-peer-disc-2
    Peer Should Not Be Connected    node-peer-disc-1    node-peer-disc-2

Get Connected Peers List
    [Tags]    network
    Connect Peers    node-peer-list-1    node-peer-list-2
    ${peers}=    Get Connected Peers    node-peer-list-1
    Should Not Be Empty    ${peers}

Node Reports Peer Count
    [Tags]    network
    Connect Peers    node-peer-count-1    node-peer-count-2
    Wait For Peer Count    node-peer-count-1    1

Node Reports Peer Via API
    [Tags]    network
    Start Node    node-peer-api-1    port=9100
    Start Node    node-peer-api-2    port=9101
    Connect Peers    node-peer-api-1    node-peer-api-2
    ${peers}=    Get Node Peers Via API
    Should Not Be Empty    ${peers}

Multiple Peers Can Connect
    [Tags]    network
    ${nodes}=    Create List    node-multi-peer-1    node-multi-peer-2    node-multi-peer-3
    Create Network Mesh    @{nodes}
    Wait For Peer Count    node-multi-peer-1    2

Connection Has Latency Metric
    [Tags]    network
    Connect Peers    node-peer-latency-1    node-peer-latency-2
    ${peers}=    Get Connected Peers    node-peer-latency-1
    ${peer}=    Set Variable    ${peers}[0]
    Dictionary Should Contain Key    ${peer}    latency_ms

Reconnect After Disconnect
    [Tags]    network
    Connect Peers    node-peer-recon-1    node-peer-recon-2
    Disconnect Peers    node-peer-recon-1    node-peer-recon-2
    Connect Peers    node-peer-recon-1    node-peer-recon-2
    Peer Should Be Connected    node-peer-recon-1    node-peer-recon-2

Connection Has Timestamp
    [Tags]    network
    Connect Peers    node-peer-ts-1    node-peer-ts-2
    ${peers}=    Get Connected Peers    node-peer-ts-1
    ${peer}=    Set Variable    ${peers}[0]
    Dictionary Should Contain Key    ${peer}    connected_at
