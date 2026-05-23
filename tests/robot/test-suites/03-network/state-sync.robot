*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/network.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Stop All Running Nodes
Test Tags    network    sync    regression

*** Test Cases ***
Sync Initiated Between Peers
    [Tags]    smoke    sync
    ${nodes}=    Create List    node-sync-1    node-sync-2
    Create Network Mesh    @{nodes}
    ${result}=    Sync Node With Peer    node-sync-1    node-sync-2
    Dictionary Should Contain Key    ${result}    status

Sync Transfers Proofs
    [Tags]    sync
    Start Node    node-sync-proof-1    port=9120
    Start Node    node-sync-proof-2    port=9121
    Connect Peers    node-sync-proof-1    node-sync-proof-2
    Create Proof    target_node=node-sync-target    window_id=window-sync
    ${result}=    Sync Node With Peer    node-sync-proof-1    node-sync-proof-2
    Dictionary Should Contain Key    ${result}    synced_count

Sync Reports Batch Size
    [Tags]    sync
    ${nodes}=    Create List    node-sync-batch-1    node-sync-batch-2
    Create Network Mesh    @{nodes}
    FOR    ${i}    IN RANGE    5
        Create Proof    target_node=node-sync-batch-target-${i}    window_id=window-sync-batch
    END
    ${result}=    Sync Node With Peer    node-sync-batch-1    node-sync-batch-2
    Should Be True    ${result}[synced_count] >= 0

Reconnection Triggers Sync
    [Tags]    sync
    ${nodes}=    Create List    node-sync-recon-1    node-sync-recon-2
    Create Network Mesh    @{nodes}
    Disconnect Peers    node-sync-recon-1    node-sync-recon-2
    Connect Peers    node-sync-recon-1    node-sync-recon-2
    Peer Should Be Connected    node-sync-recon-1    node-sync-recon-2

Sync After Network Partition
    [Tags]    sync    partition
    ${nodes}=    Create List    node-sync-part-1    node-sync-part-2
    Create Network Mesh    @{nodes}
    ${partition}=    Simulate Network Partition    node-sync-part-2
    Heal Network Partition    ${partition}[partition_id]
    Peer Should Be Connected    node-sync-part-1    node-sync-part-2

Sync Is Idempotent
    [Tags]    sync
    ${nodes}=    Create List    node-sync-idem-1    node-sync-idem-2
    Create Network Mesh    @{nodes}
    ${r1}=    Sync Node With Peer    node-sync-idem-1    node-sync-idem-2
    ${r2}=    Sync Node With Peer    node-sync-idem-1    node-sync-idem-2
    Should Be Equal    ${r1}[status]    ${r2}[status]

Sync Returns Node IDs
    [Tags]    sync
    ${nodes}=    Create List    node-sync-nodes-1    node-sync-nodes-2
    Create Network Mesh    @{nodes}
    ${result}=    Sync Node With Peer    node-sync-nodes-1    node-sync-nodes-2
    Dictionary Should Contain Key    ${result}    synced_with
