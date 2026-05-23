*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/network.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Stop All Running Nodes
Test Tags    network    multi-node    regression

*** Test Cases ***
Three Node Mesh Network
    [Tags]    smoke    network
    ${nodes}=    Create List    node-mesh-1    node-mesh-2    node-mesh-3
    Create Network Mesh    @{nodes}
    Wait For Peer Count    node-mesh-1    2

Proof Gossips Across Nodes
    [Tags]    network    gossip
    Start Node    node-gossip-1    port=9110
    Start Node    node-gossip-2    port=9111
    Connect Peers    node-gossip-1    node-gossip-2
    ${proof}=    Create Proof    target_node=node-gossip-target    window_id=window-gossip
    Sleep    2s
    Peer Should Be Connected    node-gossip-1    node-gossip-2

Five Node Full Mesh
    [Tags]    network
    ${nodes}=    Create List    node-full-1    node-full-2    node-full-3    node-full-4    node-full-5
    Create Network Mesh    @{nodes}
    FOR    ${node}    IN    @{nodes}
        Wait For Peer Count    ${node}    4
    END

Network Topology Reports All Nodes
    [Tags]    network
    ${nodes}=    Create List    node-topo-1    node-topo-2    node-topo-3
    Create Network Mesh    @{nodes}
    ${topology}=    Get Network Topology
    Should Not Be Empty    ${topology}

Chain Mesh Reduces Connections
    [Tags]    network
    ${nodes}=    Create List    node-chain-1    node-chain-2    node-chain-3    node-chain-4
    Connect Peers    node-chain-1    node-chain-2
    Connect Peers    node-chain-2    node-chain-3
    Connect Peers    node-chain-3    node-chain-4
    Wait For Peer Count    node-chain-1    1
    Wait For Peer Count    node-chain-2    2

Node Disconnects From Network
    [Tags]    network
    ${nodes}=    Create List    node-disc-net-1    node-disc-net-2    node-disc-net-3
    Create Network Mesh    @{nodes}
    Disconnect Peers    node-disc-net-1    node-disc-net-2
    Wait For Peer Count    node-disc-net-1    1

Network Stats Are Available
    [Tags]    network
    ${nodes}=    Create List    node-stats-1    node-stats-2
    Create Network Mesh    @{nodes}
    ${stats}=    Get Network Stats
    Dictionary Should Contain Key    ${stats}    total_peers

Broadcast Proof To Network
    [Tags]    network    gossip
    ${nodes}=    Create List    node-bcast-1    node-bcast-2    node-bcast-3
    Create Network Mesh    @{nodes}
    ${proof}=    Create Proof    target_node=node-bcast-target    window_id=window-bcast
    ${result}=    Broadcast Proof To Network    ${proof}[id]
    Dictionary Should Contain Key    ${result}    status
