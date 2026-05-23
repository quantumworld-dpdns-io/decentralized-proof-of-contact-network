*** Settings ***
Resource    common.resource
Resource    api.resource

*** Variables ***
${GOSSIP_FANOUT}    3
${GOSSIP_TTL}    2
${SYNC_BATCH_SIZE}    100

*** Keywords ***
Connect Peers
    [Arguments]    ${node1_id}    ${node2_id}    ${address}=${NONE}
    ${result}=    Connect Nodes    ${node1_id}    ${node2_id}    ${address}
    RETURN    ${result}

Disconnect Peers
    [Arguments]    ${node1_id}    ${node2_id}
    ${result}=    Disconnect Nodes    ${node1_id}    ${node2_id}
    RETURN    ${result}

Peer Should Be Connected
    [Arguments]    ${node1_id}    ${node2_id}
    ${peers}=    Get Connected Peers    ${node1_id}
    ${connected}=    Evaluate    any(p['node1'] == '${node2_id}' or p['node2'] == '${node2_id}' for p in ${peers})
    Should Be True    ${connected}    ${node1_id} should be connected to ${node2_id}

Peer Should Not Be Connected
    [Arguments]    ${node1_id}    ${node2_id}
    ${peers}=    Get Connected Peers    ${node1_id}
    ${connected}=    Evaluate    any(p['node1'] == '${node2_id}' or p['node2'] == '${node2_id}' for p in ${peers})
    Should Be False    ${connected}    ${node1_id} should not be connected to ${node2_id}

Create Network Mesh
    [Arguments]    @{node_ids}
    ${connections}=    Create List
    FOR    ${i}    IN RANGE    ${node_ids.__len__()}
        FOR    ${j}    IN RANGE    ${i + 1}    ${node_ids.__len__()}
            ${result}=    Connect Peers    ${node_ids}[${i}]    ${node_ids}[${j}]
            Append To List    ${connections}    ${result}
        END
    END
    RETURN    ${connections}

Broadcast Proof To Network
    [Arguments]    ${proof_id}
    ${resp}=    API Post    ${NETWORK_ENDPOINT}/broadcast/${proof_id}
    RETURN    ${resp.json()}

Sync Node With Peer
    [Arguments]    ${node_id}    ${peer_id}
    ${resp}=    API Post    ${NETWORK_ENDPOINT}/sync    json={"node_id": "${node_id}", "peer_id": "${peer_id}"}
    RETURN    ${resp.json()}

Get Network Topology
    ${resp}=    API Get    ${NETWORK_ENDPOINT}/topology
    RETURN    ${resp.json()}

Get Network Stats
    ${resp}=    API Get    ${NETWORK_ENDPOINT}/stats
    RETURN    ${resp.json()}

Wait For Peer Count
    [Arguments]    ${node_id}    ${expected_count}    ${timeout}=${POLL_TIMEOUT}
    Wait Until Keyword Succeeds With Timeout    Check Peer Count    ${timeout}    ${node_id}    ${expected_count}

Check Peer Count
    [Arguments]    ${node_id}    ${expected_count}
    ${count}=    Get Node Peer Count    ${node_id}
    Should Be Equal As Integers    ${count}    ${expected_count}

Gossip Proof Between Peers
    [Arguments]    ${proof_id}    ${from_node}    ${to_node}
    ${result}=    Broadcast Proof To Network    ${proof_id}
    Sleep    1s
    ${received}=    API Get    ${NETWORK_ENDPOINT}/gossip/check    params={"proof_id": "${proof_id}", "node_id": "${to_node}"}
    RETURN    ${received.json()}
