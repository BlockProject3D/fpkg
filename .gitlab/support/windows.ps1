function global:startsection($id, $title, $collapsed) {
    #Param([parameter(Position=0)]$id,
    #  [parameter(Position=1)]$title,
    #  [parameter(Position=2)]$collapsed)

    $date = Get-Date (Get-Date).ToUniversalTime() -UFormat %s

    if ( $collapsed -eq $null ) {
        echo "`e[0Ksection_start:${date}:${id}`r`e[0K`e[1m${title}`e[0m"
    } else {
        echo "`e[0Ksection_start:${date}:${id}[collapsed=true]`r`e[0K`e[1m${title}`e[0m"
    }
}

function global:endsection($id) {
    $date = Get-Date (Get-Date).ToUniversalTime() -UFormat %s

    echo "`e[0Ksection_end:${date}:${id}`r`e[0K"
}
