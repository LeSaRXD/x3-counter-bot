--
-- PostgreSQL database dump
--

-- Dumped from database version 14.12 (Ubuntu 14.12-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.12 (Ubuntu 14.12-0ubuntu0.22.04.1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: counter; Type: TABLE; Schema: public; Owner: x3_admin
--

CREATE TABLE public.counter (
    user_id text NOT NULL,
    emote character(2) NOT NULL,
    count integer DEFAULT 0 NOT NULL,
    CONSTRAINT counter_count_check CHECK ((count > 0))
);


ALTER TABLE public.counter OWNER TO x3_admin;

--
-- Name: opt_out; Type: TABLE; Schema: public; Owner: x3_admin
--

CREATE TABLE public.opt_out (
    user_id text NOT NULL
);


ALTER TABLE public.opt_out OWNER TO x3_admin;

--
-- Name: counter counter_pkey; Type: CONSTRAINT; Schema: public; Owner: x3_admin
--

ALTER TABLE ONLY public.counter
    ADD CONSTRAINT counter_pkey PRIMARY KEY (user_id, emote);


--
-- Name: opt_out opt_out_user_id_key; Type: CONSTRAINT; Schema: public; Owner: x3_admin
--

ALTER TABLE ONLY public.opt_out
    ADD CONSTRAINT opt_out_user_id_key UNIQUE (user_id);


--
-- PostgreSQL database dump complete
--

