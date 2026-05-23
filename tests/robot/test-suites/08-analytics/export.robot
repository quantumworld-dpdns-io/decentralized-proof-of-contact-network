*** Settings ***
Resource    ../../resources/common.resource
Resource    ../../resources/analytics.resource
Resource    ../../resources/proof.resource
Suite Setup    Setup Test Environment
Suite Teardown    Teardown Test Environment
Test Tags    analytics    export    regression

*** Test Cases ***
Export To JSON
    [Tags]    smoke    analytics
    Create Proof    target_node=node-export-json    window_id=window-export
    ${data}=    Export Analytics    proof_count_by_window    format=json
    Should Not Be Empty    ${data}

Export To Parquet
    [Tags]    analytics
    ${output}=    Set Variable    /tmp/poi-export-test.parquet
    Create Proof    target_node=node-export-parquet    window_id=window-export
    ${result}=    Export Proofs To Parquet    ${output}
    Verify Export File Exists    ${output}
    Remove File    ${output}

Export To CSV
    [Tags]    analytics
    ${output}=    Set Variable    /tmp/poi-export-test.csv
    Create Proof    target_node=node-export-csv    window_id=window-export
    ${result}=    Export Proofs To CSV    ${output}
    Verify Export File Exists    ${output}
    Remove File    ${output}

Export Contains Headers
    [Tags]    analytics
    ${output}=    Set Variable    /tmp/poi-export-headers.csv
    Create Proof    target_node=node-export-headers    window_id=window-export
    Export Proofs To CSV    ${output}
    ${content}=    Get File    ${output}
    Should Contain    ${content}    id
    Should Contain    ${content}    proving_node
    Should Contain    ${content}    target_node
    Remove File    ${output}

Export With Date Range
    [Tags]    analytics
    Create Proof    target_node=node-export-range    window_id=window-export
    ${data}=    Export Analytics    proof_count_by_window    format=json    params={"start": "2025-01-01", "end": "2025-12-31"}
    Should Not Be None    ${data}

Export Large Dataset
    [Tags]    analytics
    ${output}=    Set Variable    /tmp/poi-export-large.parquet
    FOR    ${i}    IN RANGE    20
        Create Proof    target_node=node-export-large-${i}    window_id=window-export-large
    END
    Export Proofs To Parquet    ${output}
    ${size}=    Get File Size    ${output}
    Should Be True    ${size} > 0
    Remove File    ${output}

Export With Empty Data Returns Empty
    [Tags]    analytics
    ${data}=    Export Analytics    proof_count_by_window    format=json    params={"start": "2020-01-01", "end": "2020-01-02"}
    Should Be Equal    ${data}    []

Export Format Validation
    [Tags]    analytics    negative
    Run Keyword And Expect Error    *400*    Export Analytics    proof_count_by_window    format=unsupported
