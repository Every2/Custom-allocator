             +---------------------------------------+
             |                                       |
             |           Start Execution             |
             |                                       |
             +---------------------------------------+
                            |
                            v
             +---------------------------------------+
             |                                       |
             |       Initialize IS_INIT_MALLOC       |
             |                                       |
             +------------------|--------------------+
                                |
                                v
       +------------------ Yes / No ------------------+
       |                                              |
       |      IS_INIT_MALLOC == true? (Initialized)   |
       |                                              |
       +------------------|--------------------+------+
                          |                    |
                          v                    v
             +---------------------------------------+
             |                                       |
             |         Allocate Memory (malloc)      |
             |                                       |
             +------------------|--------------------+
                                |
                                v
       +------------------ Yes / No ------------------+
       |                                              |
       |      Size <= MAX_BYTE? (Allocate on Heap)    |
       |                                              |
       +------------------|--------------------+------+
                          |                    |
                          v                    v
             +---------------------------------------+
             |                                       |
             |      Find and Allocate Chunk          |
             |         (find_chunk function)         |
             |                                       |
             +------------------|--------------------+
                                |
                                v
             +---------------------------------------+
             |                                       |
             |      Return Allocated Memory          |
             |                                       |
             +---------------------------------------+