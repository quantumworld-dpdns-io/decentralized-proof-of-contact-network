*** Settings ***
Resource    common.resource

*** Variables ***
${NODE_CONFIG_DIR}    /tmp/poi-test-nodes
${NODE_BINARY}    poi-node

*** Keywords ***
Start Node
    [Arguments]    ${node_id}    ${config_path}=${NONE}    ${port}=${NONE}
    IF    not os.path.exists(${NODE_CONFIG_DIR})    os.makedirs(${NODE_CONFIG_DIR})
    ${result}=    Start Node    ${node_id}    config_path=${config_path}    port=${port}
    Should Be Equal    ${result}[status]    running
    Log    Node ${node_id} started (PID: ${result}[pid])
    RETURN    ${result}

Stop Node
    [Arguments]    ${node_id}
    ${result}=    Stop Node    ${node_id}
    Should Be Equal    ${result}[status]    stopped
    Log    Node ${node_id} stopped
    RETURN    ${result}

Node Should Be Running
    [Arguments]    ${node_id}
    ${running}=    Node Is Running    ${node_id}
    Should Be True    ${running}    Node ${node_id} should be running

Node Should Not Be Running
    [Arguments]    ${node_id}
    ${running}=    Node Is Running    ${node_id}
    Should Be False    ${running}    Node ${node_id} should not be running

Get Node Status Info
    [Arguments]    ${node_id}
    ${info}=    Get Node Info    ${node_id}
    RETURN    ${info}

Wait For Node Ready
    [Arguments]    ${node_id}    ${timeout}=${POLL_TIMEOUT}
    Wait Until Keyword Succeeds With Timeout    Node Should Be Running    ${timeout}    ${node_id}
    Log    Node ${node_id} is ready

Connect Two Nodes
    [Arguments]    ${node1_id}    ${node2_id}
    ${result}=    Connect Nodes    ${node1_id}    ${node2_id}
    Should Be Equal    ${result}[status]    connected
    RETURN    ${result}

Disconnect Two Nodes
    [Arguments]    ${node1_id}    ${node2_id}
    ${result}=    Disconnect Nodes    ${node1_id}    ${node2_id}
    Should Be Equal    ${result}[status]    disconnected
    RETURN    ${result}

Simulate Network Partition
    [Arguments]    @{node_ids}
    ${result}=    Simulate Network Partition    ${node_ids}
    Dictionary Should Contain Key    ${result}    partition_id
    RETURN    ${result}

Heal Network Partition
    [Arguments]    ${partition_id}
    ${result}=    Heal Network Partition    ${partition_id}
    Should Be Equal    ${result}[status]    healed
    RETURN    ${result}

Stop All Running Nodes
    ${results}=    Stop All Nodes
    Log    Stopped ${results.__len__()} nodes
    RETURN    ${results}

Verify Node State
    [Arguments]    ${node_id}    ${expected_state}
    ${info}=    Get Node Status Info    ${node_id}
    Should Be Equal    ${info}[status]    ${expected_state}
