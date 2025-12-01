#!/bin/bash
echo "================================================"
echo "Test case 1: Root URL Response" 
echo "------------------------------------------------"
curl "http://localhost:8080"
echo
echo "================================================"
echo "Test case 2: Site-Wide Visit Count" 
echo "------------------------------------------------"
curl "http://localhost:8080/count"
echo
curl "http://localhost:8080/count"
echo
curl "http://localhost:8080/count"
echo
echo "================================================"
echo "Test case 3: Concurrent Visit Handling" 
echo "------------------------------------------------"
oha -n 1000000 "http://localhost:8080/count" > /dev/null
curl -s "http://localhost:8080/count"
echo
echo "================================================"
echo "Test case 4: Adding New Songs" 
echo "------------------------------------------------"
curl "http://localhost:8080/songs/new" \
  --json '{"title":"Bohemian Rhapsody", "artist":"Queen", "genre":"Rock"}'
echo
curl "http://localhost:8080/songs/new" \
  --json '{"title":"Love Story", "artist":"Taylor Swift", "genre":"Country"}'
echo
curl "http://localhost:8080/songs/new" \
  --json '{"title":"Welcome to New York", "artist":"Taylor Swift", "genre":"Pop"}'
echo
echo "================================================"
echo "Test case 5: Searching for Songs" 
echo "------------------------------------------------"
curl "http://localhost:8080/songs/search?title=Bohemian"
echo
curl "http://localhost:8080/songs/search?artist=Queen"
echo
curl "http://localhost:8080/songs/search?genre=Rock"
echo
curl "http://localhost:8080/songs/search?genre=Country&artist=Taylor+Swift"
echo
curl "http://localhost:8080/songs/search?artist=Swift"
echo
curl "http://localhost:8080/songs/search?artist=taylor+swift"
echo
curl "http://localhost:8080/songs/search?genre=Rock&artist=Taylor+Swift"
echo
echo "================================================"
echo "Test case 6: Playing Songs"
echo "------------------------------------------------"
curl "http://localhost:8080/songs/play/1"
echo
curl "http://localhost:8080/songs/play/1"
echo
curl "http://localhost:8080/songs/play/4"
echo
echo "================================================"