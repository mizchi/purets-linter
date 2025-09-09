// Error: HTTP(S) imports are not allowed

// Error: HTTPS import from CDN
import React from "https://esm.sh/react@18";

// Error: HTTP import 
import lodash from "http://unpkg.com/lodash";

// Error: HTTPS import from Deno
import { serve } from "https://deno.land/std@0.140.0/http/server.ts";

// These are OK
import fs from "fs";
import local from "./local.js";