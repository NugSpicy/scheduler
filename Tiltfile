docker_build("scheduler", "")
k8s_yaml("local.k8s.yaml")

k8s_resource(workload='scheduler', port_forwards="8080", labels="application")
k8s_resource(workload='scylla', port_forwards="9042", labels="database")